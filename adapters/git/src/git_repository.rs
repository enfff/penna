use git2::build::CheckoutBuilder;
use git2::{Cred, CredentialType, RemoteCallbacks, Repository, Signature};
use penna_core::domain::{AttachmentMeta, Entry, EntryConflict, EntryId};
use penna_core::ports::{
    AttachmentStore, ConflictView, EntryRepository, JournalClone, JournalPath, JournalSync,
    RepositoryError, SyncResult, TagCatalog,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::credentials::{
    is_https_remote, lookup_keychain_token, resolve_credentials, ResolvedCredential,
};

/// Neutral basic-auth username for HTTPS token auth (ADR 0010): servers
/// authenticate the token as password and ignore or accept any username.
const PENNA_BASIC_AUTH_USER: &str = "penna";

fn needs_callbacks(resolved: &ResolvedCredential) -> bool {
    matches!(
        resolved,
        ResolvedCredential::SshAgent | ResolvedCredential::Token(_)
    )
}

fn remote_callbacks(resolved: &ResolvedCredential) -> RemoteCallbacks<'static> {
    let token = match resolved {
        ResolvedCredential::Token(token) => Some(token.clone()),
        ResolvedCredential::SshAgent | ResolvedCredential::NoCredential => None,
    };

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |url, username_from_url, allowed| {
        if allowed.contains(CredentialType::SSH_KEY) {
            let user = username_from_url.unwrap_or("git");
            return Cred::ssh_key_from_agent(user).map_err(|e| git2::Error::new(
                git2::ErrorCode::Auth,
                git2::ErrorClass::Ssh,
                format!("ssh-agent has no key for {}: {}", url, e),
            ));
        }

        if allowed.contains(CredentialType::USER_PASS_PLAINTEXT) {
            if let Some(token) = &token {
                return Cred::userpass_plaintext(PENNA_BASIC_AUTH_USER, token);
            }
        }

        Err(git2::Error::new(
            git2::ErrorCode::Auth,
            git2::ErrorClass::Http,
            format!("no credential available for {}", url),
        ))
    });
    callbacks
}

#[derive(Clone, Copy)]
enum SyncMode {
    Smart,
    PullOnly,
    PushOnly,
}

pub struct GitJournalCloner;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TagsCatalogFile {
    tags: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct EntryTagsSidecar {
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    attachments: Vec<AttachmentMeta>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryStatus {
    pub branch: Option<String>,
    pub head_commit: Option<String>,
    pub is_dirty: bool,
    pub merge_in_progress: bool,
    pub conflicted_paths: Vec<String>,
}

#[derive(Clone)]
pub struct GitEntryRepository {
    repo: Arc<Mutex<Repository>>,
    root: PathBuf,
}

impl std::fmt::Debug for GitEntryRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitEntryRepository")
            .field("root", &self.root)
            .finish_non_exhaustive()
    }
}

impl GitEntryRepository {
    pub fn new(path: std::path::PathBuf) -> Result<Self, RepositoryError> {
        let repo_path = path.join(".git");
        
        let repo = if repo_path.exists() {
            Repository::open(&path)
                .map_err(|e| RepositoryError::Storage(format!("Failed to open git repo: {}", e)))?
        } else {
            Repository::init(&path)
                .map_err(|e| RepositoryError::Storage(format!("Failed to init git repo: {}", e)))?
        };

        Ok(Self {
            repo: Arc::new(Mutex::new(repo)),
            root: path,
        })
    }

    pub fn with_existing_repo(repo: Repository) -> Self {
        let root = repo.path().parent().map_or_else(PathBuf::new, PathBuf::from);
        Self {
            repo: Arc::new(Mutex::new(repo)),
            root,
        }
    }

    pub fn repository_path(&self) -> &PathBuf {
        &self.root
    }

    pub fn status(&self) -> Result<RepositoryStatus, RepositoryError> {
        let repo = self.repo.lock().unwrap();

        let branch = match repo.head() {
            Ok(head) if head.is_branch() => head.shorthand().map(ToOwned::to_owned),
            _ => None,
        };

        let head_commit = match repo.head() {
            Ok(head) => head
                .target()
                .map(|oid| oid.to_string()),
            Err(_) => None,
        };

        let is_dirty = !repo
            .statuses(None)
            .map_err(|e| RepositoryError::Storage(format!("Failed to get repo status: {}", e)))?
            .is_empty();

        let merge_in_progress = repo.find_reference("MERGE_HEAD").is_ok();

        let conflicted_paths = if merge_in_progress {
            let index = repo
                .index()
                .map_err(|e| RepositoryError::Storage(format!("Failed to read index: {}", e)))?;
            conflicted_paths_of(&index)
                .into_iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect()
        } else {
            Vec::new()
        };

        Ok(RepositoryStatus {
            branch,
            head_commit,
            is_dirty,
            merge_in_progress,
            conflicted_paths,
        })
    }

    fn entry_path(&self, id: &str) -> PathBuf {
        PathBuf::from(format!("{}.md", id))
    }

    fn get_head_oid(&self) -> Result<Option<git2::Oid>, RepositoryError> {
        let repo = self.repo.lock().unwrap();
        let head = repo.head();
        
        match head {
            Ok(head) if head.is_branch() => {
                let commit = head.peel_to_commit()
                    .map_err(|e| RepositoryError::Storage(format!("Failed to get head commit: {}", e)))?;
                Ok(Some(commit.id()))
            }
            _ => Ok(None),
        }
    }


    fn create_signature(&self) -> Result<Signature<'static>, RepositoryError> {
        Signature::now("Penna", "penna@example.com")
            .map_err(|e| RepositoryError::Storage(format!("Failed to create signature: {}", e)))
    }

    fn format_git_time(time: git2::Time) -> String {
        chrono::DateTime::from_timestamp(time.seconds(), 0)
            .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, false))
            .unwrap_or_default()
    }

    fn commit_touches_path(
        commit: &git2::Commit<'_>,
        path: &Path,
    ) -> Result<bool, RepositoryError> {
        let current_blob = commit
            .tree()
            .ok()
            .and_then(|tree| tree.get_path(path).ok())
            .map(|entry| entry.id());

        let parent_blob = match commit.parent_count() {
            0 => None,
            _ => match commit.parent(0) {
                Ok(parent) => parent
                    .tree()
                    .ok()
                    .and_then(|tree| tree.get_path(path).ok())
                    .map(|entry| entry.id()),
                Err(e) => {
                    return Err(RepositoryError::Storage(format!(
                        "Failed to get parent commit: {}",
                        e
                    )))
                }
            },
        };

        Ok(current_blob != parent_blob)
    }

    /// Derives durable entry timestamps from git history (ADR 0007):
    /// created_at is the author date of the earliest commit touching
    /// `<id>.md`, updated_at of the latest. Returns None when no history
    /// exists for the entry.
    fn entry_history_timestamps(
        &self,
        id: &str,
    ) -> Result<Option<(String, String)>, RepositoryError> {
        let repo = self.repo.lock().unwrap();

        if repo.head().is_err() {
            return Ok(None);
        }

        let entry_path = self.entry_path(id);
        let mut revwalk = repo
            .revwalk()
            .map_err(|e| RepositoryError::Storage(format!("Failed to start history walk: {}", e)))?;
        revwalk
            .set_sorting(git2::Sort::TIME | git2::Sort::REVERSE)
            .map_err(|e| RepositoryError::Storage(format!("Failed to sort history: {}", e)))?;
        revwalk
            .push_head()
            .map_err(|e| RepositoryError::Storage(format!("Failed to walk history: {}", e)))?;

        let mut created_at: Option<git2::Time> = None;
        let mut updated_at: Option<git2::Time> = None;

        for oid in revwalk {
            let oid = oid.map_err(|e| {
                RepositoryError::Storage(format!("Failed to read history entry: {}", e))
            })?;
            let commit = repo.find_commit(oid).map_err(|e| {
                RepositoryError::Storage(format!("Failed to find commit in history: {}", e))
            })?;

            if Self::commit_touches_path(&commit, &entry_path)? {
                if created_at.is_none() {
                    created_at = Some(commit.time());
                }
                updated_at = Some(commit.time());
            }
        }

        Ok(match (created_at, updated_at) {
            (Some(created), Some(updated)) => Some((
                Self::format_git_time(created),
                Self::format_git_time(updated),
            )),
            _ => None,
        })
    }

    fn parse_entry_content(id: &str, content: &str) -> Result<Entry, RepositoryError> {
        let timestamps = None;
        let lines: Vec<&str> = content.lines().collect();
        
        let (title, body_start) = if lines.first().map(|l| l.starts_with("# ")).unwrap_or(false) {
            (lines[0][2..].to_string(), 1)
        } else {
            ("Untitled".to_string(), 0)
        };
        
        let mut body_lines = &lines[body_start..];
        if !body_lines.is_empty() && body_lines[0].is_empty() {
            body_lines = &body_lines[1..];
        }
        let body = body_lines.join("\n");
        
        let (created_at, updated_at) = match timestamps {
            Some((c, u)) => (c, u),
            None => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| RepositoryError::Storage(format!("Failed to get timestamp: {}", e)))?
                    .as_millis()
                    .to_string();
                (now.clone(), now)
            }
        };

        Ok(Entry {
            id: EntryId(id.to_string()),
            title,
            body,
            tags: Vec::new(),
            created_at,
            updated_at,
        })
    }

    fn format_entry_content(entry: &Entry) -> String {
        format!("# {}\n\n{}", entry.title, entry.body)
    }

    fn tags_file_relative_path() -> &'static Path {
        Path::new(".penna/tags.json")
    }

    fn entry_tags_relative_path(id: &str) -> PathBuf {
        PathBuf::from(format!(".penna/{}.json", id))
    }

    fn attachment_dir(&self, id: &str) -> PathBuf {
        self.root.join(id)
    }

    fn attachment_relative_path(id: &str, name: &str) -> Result<PathBuf, RepositoryError> {
        if name.is_empty()
            || name.contains('/')
            || name.contains('\\')
            || name == "."
            || name == ".."
        {
            return Err(RepositoryError::Storage(format!(
                "invalid attachment name: {}",
                name
            )));
        }
        Ok(PathBuf::from(format!("{}/{}", id, name)))
    }

    fn tags_file_absolute_path(&self) -> PathBuf {
        self.root.join(Self::tags_file_relative_path())
    }

    fn entry_tags_absolute_path(&self, id: &str) -> PathBuf {
        self.root.join(Self::entry_tags_relative_path(id))
    }

    fn normalize_tags(mut tags: Vec<String>) -> Vec<String> {
        for tag in &mut tags {
            *tag = tag.trim().to_string();
        }
        tags.retain(|t| !t.is_empty());
        tags.sort();
        tags.dedup();
        tags
    }

    fn read_tags_from_disk(&self) -> Result<Vec<String>, RepositoryError> {
        let path = self.tags_file_absolute_path();
        if !path.exists() {
            return Ok(Vec::new());
        }

        let bytes = std::fs::read(&path).map_err(|e| {
            RepositoryError::Storage(format!("Failed to read tags file {}: {}", path.display(), e))
        })?;

        let parsed: TagsCatalogFile = serde_json::from_slice(&bytes).map_err(|e| {
            RepositoryError::Storage(format!("Failed to parse tags file {}: {}", path.display(), e))
        })?;

        Ok(Self::normalize_tags(parsed.tags))
    }

    fn read_entry_sidecar(&self, id: &str) -> Result<EntryTagsSidecar, RepositoryError> {
        let path = self.entry_tags_absolute_path(id);
        if !path.exists() {
            return Ok(EntryTagsSidecar::default());
        }

        let bytes = std::fs::read(&path).map_err(|e| {
            RepositoryError::Storage(format!(
                "Failed to read entry sidecar {}: {}",
                path.display(),
                e
            ))
        })?;

        serde_json::from_slice::<EntryTagsSidecar>(&bytes).map_err(|_| {
            RepositoryError::Storage("malformed sidecar".to_string())
        })
    }

    fn write_entry_sidecar(
        &self,
        id: &str,
        sidecar: &EntryTagsSidecar,
    ) -> Result<(), RepositoryError> {
        let file_path = self.entry_tags_absolute_path(id);

        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                RepositoryError::Storage(format!(
                    "Failed to create sidecar parent directory {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        let content = serde_json::to_vec_pretty(sidecar).map_err(|e| {
            RepositoryError::Storage(format!("Failed to encode entry sidecar: {}", e))
        })?;

        std::fs::write(&file_path, content).map_err(|e| {
            RepositoryError::Storage(format!(
                "Failed to write entry sidecar {}: {}",
                file_path.display(),
                e
            ))
        })?;

        Ok(())
    }

    fn read_entry_tags_from_disk(&self, id: &str) -> Result<Vec<String>, RepositoryError> {
        Ok(Self::normalize_tags(self.read_entry_sidecar(id)?.tags))
    }

    fn write_entry_tags_sidecar_to_disk(
        &self,
        id: &str,
        tags: Vec<String>,
    ) -> Result<(), RepositoryError> {
        let mut sidecar = self.read_entry_sidecar(id)?;
        sidecar.tags = Self::normalize_tags(tags);
        self.write_entry_sidecar(id, &sidecar)
    }

    fn list_entry_ids_from_worktree(&self) -> Result<Vec<String>, RepositoryError> {
        let mut ids = Vec::new();
        let entries = std::fs::read_dir(&self.root).map_err(|e| {
            RepositoryError::Storage(format!(
                "Failed to scan journal directory {}: {}",
                self.root.display(),
                e
            ))
        })?;
        for entry in entries {
            let path = entry.map_err(|e| {
                RepositoryError::Storage(format!("Failed to read directory entry: {}", e))
            })?;
            let name = path.file_name().to_string_lossy().into_owned();
            if let Some(stem) = name.strip_suffix(".md") {
                if path.metadata().map(|m| m.is_file()).unwrap_or(false) {
                    ids.push(stem.to_string());
                }
            }
        }
        Ok(ids)
    }

    fn list_entry_ids_from_head(&self) -> Result<Vec<String>, RepositoryError> {
        let mut entry_ids: Vec<String> = Vec::new();

        let commit_oid = match self.get_head_oid()? {
            Some(oid) => oid,
            None => return Ok(vec![]),
        };

        let repo = self.repo.lock().unwrap();

        let commit = repo
            .find_commit(commit_oid)
            .map_err(|e| RepositoryError::Storage(format!("Failed to find commit: {}", e)))?;

        let tree = commit
            .tree()
            .map_err(|e| RepositoryError::Storage(format!("Failed to get tree: {}", e)))?;

        for entry in tree.iter() {
            if entry.filemode() == git2::FileMode::Blob as i32 || entry.filemode() == 33188 {
                let path_str = entry.name().unwrap_or("");
                if path_str.ends_with(".md") {
                    let id = path_str[..path_str.len() - 3].to_string();
                    entry_ids.push(id);
                }
            }
        }

        Ok(entry_ids)
    }

    fn write_tags_to_disk_and_commit(&self, tags: Vec<String>, message: &str) -> Result<Vec<String>, RepositoryError> {
        let normalized = Self::normalize_tags(tags);
        let file_path = self.tags_file_absolute_path();

        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                RepositoryError::Storage(format!(
                    "Failed to create tags parent directory {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        let payload = TagsCatalogFile {
            tags: normalized.clone(),
        };

        let content = serde_json::to_vec_pretty(&payload).map_err(|e| {
            RepositoryError::Storage(format!("Failed to encode tags payload: {}", e))
        })?;

        std::fs::write(&file_path, content).map_err(|e| {
            RepositoryError::Storage(format!("Failed to write tags file {}: {}", file_path.display(), e))
        })?;

        let repo = self.repo.lock().unwrap();
        let mut index = repo
            .index()
            .map_err(|e| RepositoryError::Storage(format!("Failed to open git index: {}", e)))?;

        index
            .add_path(Self::tags_file_relative_path())
            .map_err(|e| RepositoryError::Storage(format!("Failed to add tags file to index: {}", e)))?;
        index
            .write()
            .map_err(|e| RepositoryError::Storage(format!("Failed to write git index: {}", e)))?;

        let tree_oid = index
            .write_tree()
            .map_err(|e| RepositoryError::Storage(format!("Failed to write git tree: {}", e)))?;
        let tree = repo
            .find_tree(tree_oid)
            .map_err(|e| RepositoryError::Storage(format!("Failed to find git tree: {}", e)))?;

        let sig = self.create_signature()?;
        let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
        let parents: Vec<&git2::Commit> = parent.as_ref().map_or_else(Vec::new, |p| vec![p]);

        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
            .map_err(|e| RepositoryError::Storage(format!("Failed to commit tags change: {}", e)))?;

        Ok(normalized)
    }

    fn sync_with_mode(&self, mode: SyncMode) -> Result<SyncResult, RepositoryError> {
        let repo = self.repo.lock().unwrap();

        let mut remote = match repo.find_remote("origin") {
            Ok(remote) => remote,
            Err(e) if e.code() == git2::ErrorCode::NotFound => return Ok(SyncResult::NoRemote),
            Err(e) => {
                return Err(RepositoryError::Storage(format!(
                    "Failed to find origin remote: {}",
                    e
                )))
            }
        };

        let remote_url = remote
            .url()
            .map(ToOwned::to_owned)
            .unwrap_or_default();

        let env_token = std::env::var("PENNA_GIT_TOKEN").ok();
        let keychain_token = if is_https_remote(&remote_url) {
            lookup_keychain_token(&remote_url)
        } else {
            None
        };
        let resolved =
            resolve_credentials(&remote_url, env_token.as_deref(), keychain_token)?;

        let branch = match repo.head() {
            Ok(head) if head.is_branch() => head
                .shorthand()
                .map(ToOwned::to_owned)
                .ok_or_else(|| RepositoryError::Storage("failed to resolve current branch".to_string()))?,
            _ => return Ok(SyncResult::NoBranch),
        };

        let local_oid = repo
            .head()
            .and_then(|head| head.peel_to_commit())
            .map_err(|e| RepositoryError::Storage(format!("Failed to get head commit: {}", e)))?
            .id();

        let fetch_refspec = format!("refs/heads/{0}:refs/remotes/origin/{0}", branch);
        let mut fetch_opts = git2::FetchOptions::new();
        if needs_callbacks(&resolved) {
            fetch_opts.remote_callbacks(remote_callbacks(&resolved));
        }
        remote
            .fetch(&[&fetch_refspec], Some(&mut fetch_opts), None)
            .map_err(|e| {
                if e.code() == git2::ErrorCode::Auth {
                    RepositoryError::AuthRequired(remote_url.clone())
                } else {
                    RepositoryError::Storage(format!("Failed to fetch remote: {}", e))
                }
            })?;

        let remote_ref_name = format!("refs/remotes/origin/{}", branch);
        let remote_oid = match repo.find_reference(&remote_ref_name) {
            Ok(reference) => reference.target().ok_or_else(|| {
                RepositoryError::Storage("remote tracking reference has no target".to_string())
            })?,
            Err(e) if e.code() == git2::ErrorCode::NotFound => {
                if matches!(mode, SyncMode::PullOnly) {
                    return Ok(SyncResult::UpToDate { branch });
                }

                let push_refspec = format!("refs/heads/{0}:refs/heads/{0}", branch);
                let mut push_opts = git2::PushOptions::new();
                if needs_callbacks(&resolved) {
                    push_opts.remote_callbacks(remote_callbacks(&resolved));
                }
                remote
                    .push(&[&push_refspec], Some(&mut push_opts))
                    .map_err(|err| {
                        if err.code() == git2::ErrorCode::Auth {
                            RepositoryError::AuthRequired(remote_url.clone())
                        } else {
                            RepositoryError::Storage(format!(
                                "Failed to push branch to remote: {}",
                                err
                            ))
                        }
                    })?;
                return Ok(SyncResult::Pushed { branch });
            }
            Err(e) => {
                return Err(RepositoryError::Storage(format!(
                    "Failed to read remote tracking branch: {}",
                    e
                )))
            }
        };

        let (ahead, behind) = repo
            .graph_ahead_behind(local_oid, remote_oid)
            .map_err(|e| RepositoryError::Storage(format!("Failed to compare histories: {}", e)))?;

        if ahead == 0 && behind == 0 {
            return Ok(SyncResult::UpToDate { branch });
        }

        if ahead > 0 && behind > 0 {
            match mode {
                SyncMode::PushOnly => {
                    return Ok(SyncResult::Diverged {
                        branch,
                        ahead,
                        behind,
                    });
                }
                SyncMode::Smart | SyncMode::PullOnly => {
                    // ADR 0014: merge (never rebase), apply marker-less
                    // policies, then conclude when nothing is conflicted.
                    self.begin_merge_locked(&repo)?;
                    self.apply_merge_policies_locked(&repo)?;
                    return match conclude_merge_locked(&repo)? {
                        MergeConclusion::Concluded => Ok(SyncResult::Pulled { branch }),
                        MergeConclusion::StillConflicted(_) => {
                            Ok(SyncResult::Diverged { branch, ahead, behind })
                        }
                        MergeConclusion::NotMerging => Ok(SyncResult::UpToDate { branch }),
                    };
                }
            }
        }

        match mode {
            SyncMode::PullOnly => {
                if behind == 0 {
                    return Ok(SyncResult::UpToDate { branch });
                }

                let local_ref_name = format!("refs/heads/{}", branch);
                let mut local_ref = repo.find_reference(&local_ref_name).map_err(|e| {
                    RepositoryError::Storage(format!("Failed to find local branch reference: {}", e))
                })?;

                local_ref.set_target(remote_oid, "penna pull fast-forward").map_err(|e| {
                    RepositoryError::Storage(format!("Failed to fast-forward branch: {}", e))
                })?;

                repo.set_head(&local_ref_name)
                    .map_err(|e| RepositoryError::Storage(format!("Failed to set HEAD: {}", e)))?;

                repo.checkout_head(Some(CheckoutBuilder::new().force()))
                    .map_err(|e| RepositoryError::Storage(format!("Failed to checkout updated HEAD: {}", e)))?;

                Ok(SyncResult::Pulled { branch })
            }
            SyncMode::PushOnly => {
                if ahead == 0 || behind > 0 {
                    return Ok(SyncResult::Diverged {
                        branch,
                        ahead,
                        behind,
                    });
                }

                let push_refspec = format!("refs/heads/{0}:refs/heads/{0}", branch);
                let mut push_opts = git2::PushOptions::new();
                if needs_callbacks(&resolved) {
                    push_opts.remote_callbacks(remote_callbacks(&resolved));
                }
                remote
                    .push(&[&push_refspec], Some(&mut push_opts))
                    .map_err(|e| {
                        if e.code() == git2::ErrorCode::Auth {
                            RepositoryError::AuthRequired(remote_url.clone())
                        } else {
                            RepositoryError::Storage(format!("Failed to push local commits: {}", e))
                        }
                    })?;

                Ok(SyncResult::Pushed { branch })
            }
            SyncMode::Smart => {
                if behind > 0 {
                    let local_ref_name = format!("refs/heads/{}", branch);
                    let mut local_ref = repo.find_reference(&local_ref_name).map_err(|e| {
                        RepositoryError::Storage(format!("Failed to find local branch reference: {}", e))
                    })?;

                    local_ref.set_target(remote_oid, "penna sync fast-forward").map_err(|e| {
                        RepositoryError::Storage(format!("Failed to fast-forward branch: {}", e))
                    })?;

                    repo.set_head(&local_ref_name)
                        .map_err(|e| RepositoryError::Storage(format!("Failed to set HEAD: {}", e)))?;

                    repo.checkout_head(Some(CheckoutBuilder::new().force()))
                        .map_err(|e| RepositoryError::Storage(format!("Failed to checkout updated HEAD: {}", e)))?;

                    return Ok(SyncResult::Pulled { branch });
                }

                let push_refspec = format!("refs/heads/{0}:refs/heads/{0}", branch);
                let mut push_opts = git2::PushOptions::new();
                if needs_callbacks(&resolved) {
                    push_opts.remote_callbacks(remote_callbacks(&resolved));
                }
                remote
                    .push(&[&push_refspec], Some(&mut push_opts))
                    .map_err(|e| {
                        if e.code() == git2::ErrorCode::Auth {
                            RepositoryError::AuthRequired(remote_url.clone())
                        } else {
                            RepositoryError::Storage(format!("Failed to push local commits: {}", e))
                        }
                    })?;

                Ok(SyncResult::Pushed { branch })
            }
        }
    }
}

impl JournalClone for GitJournalCloner {
    fn clone_journal(&self, remote_url: &str, local_path: &PathBuf) -> Result<(), RepositoryError> {
        Repository::clone(remote_url, local_path).map_err(|e| {
            RepositoryError::Storage(format!(
                "Failed to clone repository from {} to {}: {}",
                remote_url,
                local_path.display(),
                e
            ))
        })?;

        Ok(())
    }
}

impl JournalPath for GitEntryRepository {
    fn resolve_path(&self) -> Result<PathBuf, RepositoryError> {
        self.root
            .canonicalize()
            .map_err(|e| RepositoryError::Storage(format!("Failed to canonicalize repo path: {}", e)))
    }
}

impl TagCatalog for GitEntryRepository {
    fn list_tags(&self) -> Result<Vec<String>, RepositoryError> {
        self.read_tags_from_disk()
    }

    fn add_tag(&self, tag: &str) -> Result<Vec<String>, RepositoryError> {
        let mut tags = self.read_tags_from_disk()?;
        if !tags.iter().any(|t| t == tag) {
            tags.push(tag.to_string());
        }
        self.write_tags_to_disk_and_commit(tags, &format!("Add tag {}", tag))
    }

    fn remove_tag(&self, tag: &str) -> Result<Vec<String>, RepositoryError> {
        let mut tags = self.read_tags_from_disk()?;
        tags.retain(|t| t != tag);
        let updated = self.write_tags_to_disk_and_commit(tags, &format!("Remove tag {}", tag))?;

        for id in self.list_entry_ids_from_head()? {
            let mut entry_tags = self.read_entry_tags_from_disk(&id)?;
            entry_tags.retain(|t| t != tag);
            self.write_entry_tags_sidecar_to_disk(&id, entry_tags)?;
        }

        Ok(updated)
    }

    fn update_tag(&self, old_tag: &str, new_tag: &str) -> Result<Vec<String>, RepositoryError> {
        let mut tags = self.read_tags_from_disk()?;
        let Some(position) = tags.iter().position(|t| t == old_tag) else {
            return Err(RepositoryError::NotFound(old_tag.to_string()));
        };

        tags[position] = new_tag.to_string();
        let updated = self.write_tags_to_disk_and_commit(
            tags,
            &format!("Rename tag {} to {}", old_tag, new_tag),
        )?;

        for id in self.list_entry_ids_from_head()? {
            let mut entry_tags = self.read_entry_tags_from_disk(&id)?;
            for value in &mut entry_tags {
                if value == old_tag {
                    *value = new_tag.to_string();
                }
            }
            self.write_entry_tags_sidecar_to_disk(&id, entry_tags)?;
        }

        Ok(updated)
    }
}

impl EntryRepository for GitEntryRepository {
    /// Reads the entry from the working tree (ADR 0014): external edits and
    /// conflict markers must surface immediately. Timestamps still derive
    /// from git history.
    fn get(&self, id: &str) -> Result<Option<Entry>, RepositoryError> {
        let absolute = self.root.join(self.entry_path(id));
        if !absolute.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&absolute).map_err(|e| {
            RepositoryError::Storage(format!(
                "Failed to read entry file {}: {}",
                absolute.display(),
                e
            ))
        })?;

        let mut entry = Self::parse_entry_content(id, &content)?;
        entry.tags = self.read_entry_tags_from_disk(id)?;
        if let Some((created_at, updated_at)) = self.entry_history_timestamps(id)? {
            entry.created_at = created_at;
            entry.updated_at = updated_at;
        }
        Ok(Some(entry))
    }

    fn save(&self, entry: &Entry) -> Result<(), RepositoryError> {
        let entry_path = self.entry_path(&entry.id.0);
        let content = Self::format_entry_content(entry);
        let sig = self.create_signature()?;
        let absolute_entry_path = self.root.join(&entry_path);

        std::fs::write(&absolute_entry_path, content.as_bytes()).map_err(|e| {
            RepositoryError::Storage(format!(
                "Failed to write entry file {}: {}",
                absolute_entry_path.display(),
                e
            ))
        })?;

        self.write_entry_tags_sidecar_to_disk(&entry.id.0, entry.tags.clone())?;

        let repo = self.repo.lock().unwrap();
        let mut index = repo
            .index()
            .map_err(|e| RepositoryError::Storage(format!("Failed to open git index: {}", e)))?;

        index
            .add_path(&entry_path)
            .map_err(|e| RepositoryError::Storage(format!("Failed to add entry to index: {}", e)))?;
        index
            .add_path(&Self::entry_tags_relative_path(&entry.id.0))
            .map_err(|e| RepositoryError::Storage(format!("Failed to add sidecar to index: {}", e)))?;
        index
            .write()
            .map_err(|e| RepositoryError::Storage(format!("Failed to write git index: {}", e)))?;

        if merge_head_oid(&repo)?.is_some() {
            // Mid-merge (ADR 0014): staging registers the resolution; the
            // conclude step creates the two-parent commit.
            return Ok(());
        }

        let tree_oid = index
            .write_tree()
            .map_err(|e| RepositoryError::Storage(format!("Failed to write git tree: {}", e)))?;
        let tree = repo
            .find_tree(tree_oid)
            .map_err(|e| RepositoryError::Storage(format!("Failed to find git tree: {}", e)))?;

        let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
        let parents: Vec<&git2::Commit> = parent.as_ref().map_or_else(Vec::new, |p| vec![p]);

        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &format!("{} entry {}", if parents.is_empty() { "Create" } else { "Update" }, entry.id.0),
            &tree,
            &parents,
        )
        .map_err(|e| RepositoryError::Storage(format!("Failed to commit: {}", e)))?;

        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), RepositoryError> {
        let repo = self.repo.lock().unwrap();
        let entry_path = self.entry_path(id);
        let entry_tags_path = Self::entry_tags_relative_path(id);
        let absolute_entry_path = self.root.join(&entry_path);
        let absolute_entry_tags_path = self.root.join(&entry_tags_path);

        let attachment_dir = self.attachment_dir(id);
        if attachment_dir.exists() {
            std::fs::remove_dir_all(&attachment_dir).map_err(|e| {
                RepositoryError::Storage(format!(
                    "Failed to remove attachment directory {}: {}",
                    attachment_dir.display(),
                    e
                ))
            })?;
        }

        if absolute_entry_path.exists() {
            std::fs::remove_file(&absolute_entry_path).map_err(|e| {
                RepositoryError::Storage(format!(
                    "Failed to remove entry file {}: {}",
                    absolute_entry_path.display(),
                    e
                ))
            })?;
        }

        if absolute_entry_tags_path.exists() {
            std::fs::remove_file(&absolute_entry_tags_path).map_err(|e| {
                RepositoryError::Storage(format!(
                    "Failed to remove sidecar file {}: {}",
                    absolute_entry_tags_path.display(),
                    e
                ))
            })?;
        }

        let sig = self.create_signature()?;
        let mut index = repo
            .index()
            .map_err(|e| RepositoryError::Storage(format!("Failed to open git index: {}", e)))?;

        index
            .remove_path(&entry_path)
            .map_err(|e| RepositoryError::Storage(format!("Failed to remove entry from index: {}", e)))?;

        if attachment_dir.exists() || index.get_path(&PathBuf::from(id), 0).is_some() {
            let spec = format!("{}/*", id);
            let _ = index.remove_all([&spec], None);
        }

        if self.root.join(&entry_tags_path).exists() {
            index.remove_path(&entry_tags_path).map_err(|e| {
                RepositoryError::Storage(format!("Failed to remove sidecar from index: {}", e))
            })?;
        } else {
            let _ = index.remove_path(&entry_tags_path);
        }

        index
            .write()
            .map_err(|e| RepositoryError::Storage(format!("Failed to write git index: {}", e)))?;

        if merge_head_oid(&repo)?.is_some() {
            // Mid-merge (ADR 0014): stage the deletion, conclude later.
            return Ok(());
        }

        let tree_oid = index
            .write_tree()
            .map_err(|e| RepositoryError::Storage(format!("Failed to write git tree: {}", e)))?;
        let tree = repo
            .find_tree(tree_oid)
            .map_err(|e| RepositoryError::Storage(format!("Failed to find git tree: {}", e)))?;

        let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
        let parents: Vec<&git2::Commit> = parent.as_ref().map_or_else(Vec::new, |p| vec![p]);

        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &format!("Delete entry {}", id),
            &tree,
            &parents,
        )
        .map_err(|e| RepositoryError::Storage(format!("Failed to commit: {}", e)))?;

        Ok(())
    }

    fn list(&self) -> Result<Vec<Entry>, RepositoryError> {
        let mut entry_ids = self.list_entry_ids_from_worktree()?;

        if entry_ids.is_empty() {
            entry_ids = self.list_entry_ids_from_head()?;
        }

        let mut entries = Vec::new();
        for id in entry_ids {
            if let Ok(Some(entry)) = self.get(&id) {
                entries.push(entry);
            }
        }

        entries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(entries)
    }
}

impl AttachmentStore for GitEntryRepository {
    fn list_attachments(&self, id: &str) -> Result<Vec<AttachmentMeta>, RepositoryError> {
        match self.read_entry_sidecar(id) {
            Ok(sidecar) => {
                let mut list = sidecar.attachments;
                list.sort_by(|a, b| a.name.cmp(&b.name));
                Ok(list)
            }
            Err(_) => self.scan_attachment_dir(id),
        }
    }

    fn get_attachment(&self, id: &str, name: &str) -> Result<Option<Vec<u8>>, RepositoryError> {
        let path = Self::attachment_relative_path(id, name)?;
        let absolute = self.root.join(&path);
        if !absolute.exists() {
            return Ok(None);
        }
        std::fs::read(&absolute)
            .map(Some)
            .map_err(|e| {
                RepositoryError::Storage(format!(
                    "Failed to read attachment {}: {}",
                    absolute.display(),
                    e
                ))
            })
    }

    fn add_attachment(
        &self,
        id: &str,
        name: &str,
        data: &[u8],
    ) -> Result<AttachmentMeta, RepositoryError> {
        let relative = Self::attachment_relative_path(id, name)?;
        if self.get(id)?.is_none() {
            return Err(RepositoryError::NotFound(id.to_string()));
        }

        let dir = self.attachment_dir(id);
        std::fs::create_dir_all(&dir).map_err(|e| {
            RepositoryError::Storage(format!(
                "Failed to create attachment directory {}: {}",
                dir.display(),
                e
            ))
        })?;

        let absolute = self.root.join(&relative);
        std::fs::write(&absolute, data).map_err(|e| {
            RepositoryError::Storage(format!(
                "Failed to write attachment {}: {}",
                absolute.display(),
                e
            ))
        })?;

        let mut sidecar = self.read_entry_sidecar(id)?;
        sidecar.attachments.retain(|meta| meta.name != name);
        sidecar.attachments.push(AttachmentMeta {
            name: name.to_string(),
            bytes: data.len() as u64,
        });
        sidecar.attachments.sort_by(|a, b| a.name.cmp(&b.name));
        self.write_entry_sidecar(id, &sidecar)?;

        self.commit_paths(
            &[relative, Self::entry_tags_relative_path(id)],
            &format!("Add attachment {} to {}", name, id),
        )?;

        Ok(AttachmentMeta {
            name: name.to_string(),
            bytes: data.len() as u64,
        })
    }

    fn remove_attachment(
        &self,
        id: &str,
        name: &str,
    ) -> Result<Vec<AttachmentMeta>, RepositoryError> {
        let relative = Self::attachment_relative_path(id, name)?;
        let mut sidecar = self.read_entry_sidecar(id)?;
        if !sidecar.attachments.iter().any(|meta| meta.name == name) {
            return Err(RepositoryError::NotFound(name.to_string()));
        }

        let absolute = self.root.join(&relative);
        if absolute.exists() {
            std::fs::remove_file(&absolute).map_err(|e| {
                RepositoryError::Storage(format!(
                    "Failed to remove attachment {}: {}",
                    absolute.display(),
                    e
                ))
            })?;
        }

        sidecar.attachments.retain(|meta| meta.name != name);
        self.write_entry_sidecar(id, &sidecar)?;

        let _ = std::fs::remove_dir(self.attachment_dir(id));

        self.commit_staged(
            &[Self::entry_tags_relative_path(id)],
            &[relative],
            &format!("Remove attachment {} from {}", name, id),
        )?;

        Ok(sidecar.attachments)
    }
}

impl GitEntryRepository {
    fn scan_attachment_dir(&self, id: &str) -> Result<Vec<AttachmentMeta>, RepositoryError> {
        let dir = self.attachment_dir(id);
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut metas = Vec::new();
        for file in std::fs::read_dir(&dir).map_err(|e| {
            RepositoryError::Storage(format!(
                "Failed to scan attachment directory {}: {}",
                dir.display(),
                e
            ))
        })? {
            let entry = file.map_err(|e| {
                RepositoryError::Storage(format!("Failed to read directory entry: {}", e))
            })?;
            let meta = entry.metadata().map_err(|e| {
                RepositoryError::Storage(format!("Failed to stat {}: {}", entry.path().display(), e))
            })?;
            if !meta.is_file() {
                continue;
            }
            metas.push(AttachmentMeta {
                name: entry.file_name().to_string_lossy().into_owned(),
                bytes: meta.len(),
            });
        }

        metas.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(metas)
    }

    /// Stages the given repo-relative paths (adds first, removals after)
    /// and commits them on HEAD.
    fn commit_paths(&self, paths: &[PathBuf], message: &str) -> Result<(), RepositoryError> {
        self.commit_staged(paths, &[], message)
    }

    fn commit_staged(
        &self,
        adds: &[PathBuf],
        removals: &[PathBuf],
        message: &str,
    ) -> Result<(), RepositoryError> {
        let repo = self.repo.lock().unwrap();
        let mut index = repo
            .index()
            .map_err(|e| RepositoryError::Storage(format!("Failed to open git index: {}", e)))?;

        for path in adds {
            index.add_path(path).map_err(|e| {
                RepositoryError::Storage(format!("Failed to add {} to index: {}", path.display(), e))
            })?;
        }
        for path in removals {
            let _ = index.remove_path(path);
        }
        index
            .write()
            .map_err(|e| RepositoryError::Storage(format!("Failed to write git index: {}", e)))?;

        let tree_oid = index
            .write_tree()
            .map_err(|e| RepositoryError::Storage(format!("Failed to write git tree: {}", e)))?;
        let tree = repo
            .find_tree(tree_oid)
            .map_err(|e| RepositoryError::Storage(format!("Failed to find git tree: {}", e)))?;

        let sig = self.create_signature()?;
        let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
        let parents: Vec<&git2::Commit> = parent.as_ref().map_or_else(Vec::new, |p| vec![p]);

        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
            .map_err(|e| RepositoryError::Storage(format!("Failed to commit: {}", e)))?;

        Ok(())
    }
}

impl JournalSync for GitEntryRepository {
    fn sync(&self) -> Result<SyncResult, RepositoryError> {
        self.sync_with_mode(SyncMode::Smart)
    }

    fn pull(&self) -> Result<SyncResult, RepositoryError> {
        self.sync_with_mode(SyncMode::PullOnly)
    }

    fn push(&self) -> Result<SyncResult, RepositoryError> {
        self.sync_with_mode(SyncMode::PushOnly)
    }
}


impl ConflictView for GitEntryRepository {
    /// Conflicted entry ids straight from the on-disk index (ADR 0014).
    fn list_conflicted_ids(&self) -> Result<Vec<String>, RepositoryError> {
        let repo = self.repo.lock().unwrap();
        let index = repo
            .index()
            .map_err(|e| RepositoryError::Storage(format!("Failed to read index: {}", e)))?;
        if !index.has_conflicts() {
            return Ok(Vec::new());
        }
        let mut ids = Vec::new();
        for path in conflicted_paths_of(&index) {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if let Some(stem) = name.strip_suffix(".md") {
                    ids.push(stem.to_string());
                }
            }
        }
        ids.sort();
        Ok(ids)
    }

    /// Structured three-way view served from index stages mid-merge.
    fn entry_conflict(&self, id: &str) -> Result<Option<EntryConflict>, RepositoryError> {
        let repo = self.repo.lock().unwrap();
        let index = repo
            .index()
            .map_err(|e| RepositoryError::Storage(format!("Failed to read index: {}", e)))?;
        if !index.has_conflicts() {
            return Ok(None);
        }
        let path = self.entry_path(id);
        let stage = |s: i32| -> Option<String> {
            let entry = index.get_path(&path, s)?;
            let blob = repo.find_blob(entry.id).ok()?;
            String::from_utf8(blob.content().to_vec()).ok()
        };
        let (Some(base), Some(ours), Some(theirs)) = (stage(1), stage(2), stage(3)) else {
            return Ok(None);
        };
        Ok(Some(EntryConflict {
            entry_id: id.to_string(),
            base,
            ours,
            theirs,
        }))
    }

    /// Concludes the pending merge when no conflicts remain (ADR 0014).
    fn reconcile_with_remote(&self) -> Result<(), RepositoryError> {
        let repo = self.repo.lock().unwrap();
        match conclude_merge_locked(&repo)? {
            MergeConclusion::Concluded | MergeConclusion::NotMerging => Ok(()),
            MergeConclusion::StillConflicted(paths) => Err(RepositoryError::Storage(format!(
                "unresolved conflicts remain: {}",
                paths.join(", ")
            ))),
        }
    }
}

pub(crate) enum MergeConclusion {
    NotMerging,
    StillConflicted(Vec<String>),
    Concluded,
}

fn merge_head_oid(repo: &git2::Repository) -> Result<Option<git2::Oid>, RepositoryError> {
    match repo.find_reference("MERGE_HEAD") {
        Ok(reference) => reference.target().map(Some).ok_or_else(|| {
            RepositoryError::Storage("MERGE_HEAD has no target".to_string())
        }),
        Err(_) => Ok(None),
    }
}

fn conclude_merge_locked(
    repo: &git2::Repository,
) -> Result<MergeConclusion, RepositoryError> {
    if merge_head_oid(repo)?.is_none() {
        return Ok(MergeConclusion::NotMerging);
    }

    let mut index = repo
        .index()
        .map_err(|e| RepositoryError::Storage(format!("Failed to read index: {}", e)))?;
    if index.has_conflicts() {
        return Ok(MergeConclusion::StillConflicted(
            conflicted_paths_of(&index)
                .into_iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect(),
        ));
    }

    let tree_oid = index
        .write_tree_to(repo)
        .map_err(|e| RepositoryError::Storage(format!("Failed to write merge tree: {}", e)))?;
    let tree = repo
        .find_tree(tree_oid)
        .map_err(|e| RepositoryError::Storage(format!("Failed to find merge tree: {}", e)))?;

    let branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(ToOwned::to_owned))
        .ok_or_else(|| RepositoryError::Storage("failed to resolve branch".to_string()))?;
    let ours = repo
        .head()
        .and_then(|h| h.peel_to_commit())
        .map_err(|e| RepositoryError::Storage(format!("Failed to get head commit: {}", e)))?;
    let merge_head_oid = merge_head_oid(repo)?.expect("checked above");
    let merge_head = repo.find_commit(merge_head_oid).map_err(|e| {
        RepositoryError::Storage(format!("Failed to find MERGE_HEAD commit: {}", e))
    })?;

    let sig = Signature::now("Penna", "penna@example.com")
        .map_err(|e| RepositoryError::Storage(format!("Failed to create signature: {}", e)))?;

    repo.commit(
        Some(&format!("refs/heads/{}", branch)),
        &sig,
        &sig,
        "Reconcile journal divergence",
        &tree,
        &[&ours, &merge_head],
    )
    .map_err(|e| RepositoryError::Storage(format!("Failed to commit merge: {}", e)))?;

    if let Ok(mut merge_head_ref) = repo.find_reference("MERGE_HEAD") {
        let _ = merge_head_ref.delete();
    }
    if let Ok(mut merge_msg_ref) = repo.find_reference("MERGE_MSG") {
        let _ = merge_msg_ref.delete();
    }

    repo.checkout_head(Some(CheckoutBuilder::new().force()))
        .map_err(|e| {
            RepositoryError::Storage(format!("Failed to refresh working tree: {}", e))
        })?;

    Ok(MergeConclusion::Concluded)
}

impl GitEntryRepository {
    /// Starts a real git merge against the fetched remote branch (ADR 0014).
    /// No-op when a merge is already in progress. Pins the marker style.
    fn begin_merge_locked(&self, repo: &git2::Repository) -> Result<(), RepositoryError> {
        if merge_head_oid(repo)?.is_some() {
            return Ok(());
        }

        let mut config = repo
            .config()
            .map_err(|e| RepositoryError::Storage(format!("Failed to read config: {}", e)))?;
        config
            .set_str("merge.conflictStyle", "merge")
            .map_err(|e| RepositoryError::Storage(format!("Failed to pin conflict style: {}", e)))?;

        let branch = repo
            .head()
            .ok()
            .and_then(|h| h.shorthand().map(ToOwned::to_owned))
            .ok_or_else(|| RepositoryError::Storage("failed to resolve branch".to_string()))?;
        let remote_ref_name = format!("refs/remotes/origin/{}", branch);
        let remote_ref = repo
            .find_reference(&remote_ref_name)
            .map_err(|e| {
                RepositoryError::Storage(format!("Failed to find remote branch: {}", e))
            })?;
        let annotated = repo
            .reference_to_annotated_commit(&remote_ref)
            .map_err(|e| {
                RepositoryError::Storage(format!("Failed to resolve remote commit: {}", e))
            })?;

        let mut merge_opts = git2::MergeOptions::new();
        let mut checkout = CheckoutBuilder::new();
        checkout.allow_conflicts(true);
        repo.merge(&[&annotated], Some(&mut merge_opts), Some(&mut checkout))
            .map_err(|e| RepositoryError::Storage(format!("Failed to merge remote: {}", e)))?;

        Ok(())
    }

    /// Resolves marker-less conflicts right after merge start: sidecars
    /// union-merge, modified sides resurrect, both-deleted drops (ADR 0014).
    fn apply_merge_policies_locked(&self, repo: &git2::Repository) -> Result<(), RepositoryError> {
        let mut index = repo
            .index()
            .map_err(|e| RepositoryError::Storage(format!("Failed to read index: {}", e)))?;
        if !index.has_conflicts() {
            return Ok(());
        }

        for path in conflicted_paths_of(&index) {
            let has_ours = index.get_path(&path, 2).is_some();
            let has_theirs = index.get_path(&path, 3).is_some();

            if path.starts_with(".penna/") {
                let merged = union_stage_tags(repo, &index, &path)?;
                std::fs::write(self.root.join(&path), merged.as_bytes()).map_err(|e| {
                    RepositoryError::Storage(format!(
                        "Failed to write merged tags {}: {}",
                        path.display(),
                        e
                    ))
                })?;
                index.add_path(&path).map_err(|e| {
                    RepositoryError::Storage(format!(
                        "Failed to stage merged tags {}: {}",
                        path.display(),
                        e
                    ))
                })?;
                continue;
            }

            match (has_ours, has_theirs) {
                (true, false) => {
                    // Theirs deleted what we modified: ours wins in place.
                    index.add_path(&path).map_err(|e| {
                        RepositoryError::Storage(format!(
                            "Failed to keep local {}: {}",
                            path.display(),
                            e
                        ))
                    })?;
                }
                (false, true) => {
                    // We deleted what they modified: resurrect theirs.
                    let content = stage_content(repo, &index, &path, 3).ok_or_else(|| {
                        RepositoryError::Storage(format!(
                            "Failed to read their side of {}",
                            path.display()
                        ))
                    })?;
                    std::fs::write(self.root.join(&path), content.as_bytes()).map_err(|e| {
                        RepositoryError::Storage(format!(
                            "Failed to resurrect {}: {}",
                            path.display(),
                            e
                        ))
                    })?;
                    index.add_path(&path).map_err(|e| {
                        RepositoryError::Storage(format!(
                            "Failed to stage resurrected {}: {}",
                            path.display(),
                            e
                        ))
                    })?;
                }
                (false, false) => {
                    // Both deleted: drop silently.
                    if index.get_path(&path, 1).is_some() {
                        let _ = index.remove(&path, 1);
                    }
                }
                (true, true) => {}
            }
        }

        index
            .write()
            .map_err(|e| RepositoryError::Storage(format!("Failed to write index: {}", e)))?;

        Ok(())
    }
}

fn stage_content(
    repo: &git2::Repository,
    index: &git2::Index,
    path: &Path,
    stage: i32,
) -> Option<String> {
    let entry = index.get_path(path, stage)?;
    let blob = repo.find_blob(entry.id).ok()?;
    String::from_utf8(blob.content().to_vec()).ok()
}

fn union_stage_tags(
    repo: &git2::Repository,
    index: &git2::Index,
    path: &Path,
) -> Result<String, RepositoryError> {
    let parse = |stage: i32| -> Vec<String> {
        stage_content(repo, index, path, stage)
            .and_then(|text| serde_json::from_str::<TagsCatalogFile>(&text).ok())
            .map(|file| file.tags)
            .unwrap_or_default()
    };

    let mut tags = parse(3);
    for tag in parse(2) {
        if !tags.contains(&tag) {
            tags.push(tag);
        }
    }
    tags.sort();
    tags.dedup();

    serde_json::to_string_pretty(&TagsCatalogFile { tags })
        .map_err(|e| RepositoryError::Storage(format!("Failed to encode merged tag file: {}", e)))
}


fn conflicted_paths_of(index: &git2::Index) -> Vec<PathBuf> {    let mut paths: Vec<PathBuf> = Vec::new();
    let Ok(conflicts) = index.conflicts() else {
        return paths;
    };
    for conflict in conflicts.flatten() {
        for entry in [conflict.ancestor, conflict.our, conflict.their]
            .into_iter()
            .flatten()
        {
            let path = PathBuf::from(String::from_utf8_lossy(&entry.path).into_owned());
            if !paths.contains(&path) {
                paths.push(path);
            }
        }
    }
    paths
}

#[cfg(test)]
mod tests {
    use super::*;
    use penna_core::ports::{JournalClone, JournalPath, JournalSync};
    use tempfile::TempDir;

    fn create_test_repo() -> (TempDir, GitEntryRepository) {
        let tmp_dir = TempDir::new().unwrap();
        let repo = GitEntryRepository::new(tmp_dir.path().to_path_buf()).unwrap();
        (tmp_dir, repo)
    }

    fn add_origin_remote(repo: &GitEntryRepository, remote_path: &std::path::Path) {
        let repo_lock = repo.repo.lock().unwrap();
        repo_lock
            .remote("origin", remote_path.to_str().unwrap())
            .unwrap();
    }

    #[test]
    fn test_create_and_get_entry() {
        let (tmp_dir, repo) = create_test_repo();
        
        let entry = Entry {
            id: EntryId("test-1".to_string()),
            title: "Test Entry".to_string(),
            body: "Test body content".to_string(),
            tags: vec![],
            created_at: "123".to_string(),
            updated_at: "123".to_string(),
        };

        repo.save(&entry).unwrap();

        let file_path = tmp_dir.path().join("test-1.md");
        assert!(file_path.exists());
        let file_content = std::fs::read_to_string(&file_path).unwrap();
        assert!(file_content.starts_with("# Test Entry\n\n"));
        assert!(file_content.contains("Test body content"));
        
        let retrieved = repo.get("test-1").unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.title, "Test Entry");
        assert_eq!(retrieved.body, "Test body content");
    }

    #[test]
    fn test_delete_removes_working_tree_file() {
        let (tmp_dir, repo) = create_test_repo();

        let entry = Entry {
            id: EntryId("test-delete".to_string()),
            title: "Delete Me".to_string(),
            body: "Body".to_string(),
            tags: vec![],
            created_at: "123".to_string(),
            updated_at: "123".to_string(),
        };

        repo.save(&entry).unwrap();
        let file_path = tmp_dir.path().join("test-delete.md");
        assert!(file_path.exists());

        repo.delete("test-delete").unwrap();
        assert!(!file_path.exists());
    }

    #[test]
    fn test_list_entries() {
        let (_tmp_dir, repo) = create_test_repo();
        
        let entry1 = Entry {
            id: EntryId("test-1".to_string()),
            title: "Entry 1".to_string(),
            body: "Body 1".to_string(),
            tags: vec![],
            created_at: "100".to_string(),
            updated_at: "100".to_string(),
        };

        let entry2 = Entry {
            id: EntryId("test-2".to_string()),
            title: "Entry 2".to_string(),
            body: "Body 2".to_string(),
            tags: vec![],
            created_at: "200".to_string(),
            updated_at: "200".to_string(),
        };

        repo.save(&entry1).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        repo.save(&entry2).unwrap();
        
        let entries = repo.list().unwrap();
        assert_eq!(entries.len(), 2);
        let titles: Vec<&str> = entries.iter().map(|e| e.title.as_str()).collect();
        assert!(titles.contains(&"Entry 1"));
        assert!(titles.contains(&"Entry 2"));
    }

    #[test]
    fn test_entry_tags_persist_in_sidecar() {
        let (_tmp_dir, repo) = create_test_repo();

        let entry = Entry {
            id: EntryId("test-tags".to_string()),
            title: "Tagged".to_string(),
            body: "Tagged body".to_string(),
            tags: vec!["work".to_string(), "daily-note".to_string()],
            created_at: "2026-08-09T10:00:00+00:00".to_string(),
            updated_at: "2026-08-09T11:00:00+00:00".to_string(),
        };

        repo.save(&entry).unwrap();

        let loaded = repo.get("test-tags").unwrap().unwrap();
        assert_eq!(
            loaded.tags,
            vec!["daily-note".to_string(), "work".to_string()]
        );
    }

    #[test]
    fn test_plain_markdown_reads() {
        let content = "# Legacy Title\n\nLegacy body";

        let parsed = GitEntryRepository::parse_entry_content("legacy-id", content).unwrap();

        assert_eq!(parsed.id.0, "legacy-id");
        assert_eq!(parsed.title, "Legacy Title");
        assert_eq!(parsed.body, "Legacy body");
    }

    #[test]
    fn test_get_derives_timestamps_from_git_history() {
        let (_tmp_dir, repo) = create_test_repo();

        repo.save(&Entry {
            id: EntryId("202608241200".to_string()),
            title: "First".to_string(),
            body: "Original body".to_string(),
            tags: vec![],
            created_at: "unused".to_string(),
            updated_at: "unused".to_string(),
        })
        .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));

        let mut updated_entry = repo.get("202608241200").unwrap().unwrap();
        updated_entry.body = "Rewritten body".to_string();
        repo.save(&updated_entry).unwrap();

        let loaded = repo.get("202608241200").unwrap().unwrap();

        assert_eq!(loaded.title, "First");
        assert_eq!(loaded.body, "Rewritten body");

        for stamp in [&loaded.created_at, &loaded.updated_at] {
            assert!(
                chrono::DateTime::parse_from_rfc3339(stamp).is_ok(),
                "timestamp {} is not RFC3339",
                stamp
            );
        }

        assert_ne!(
            loaded.created_at, loaded.updated_at,
            "update commit must move updated_at past created_at"
        );
        assert!(loaded.created_at < loaded.updated_at);
    }

    #[test]
    fn test_timestamps_survive_clone_to_another_machine() {
        let remote_dir = TempDir::new().unwrap();
        Repository::init_bare(remote_dir.path()).unwrap();

        let (_tmp_a, repo_a) = create_test_repo();
        add_origin_remote(&repo_a, remote_dir.path());
        repo_a
            .save(&Entry {
                id: EntryId("202608241300".to_string()),
                title: "Traveler".to_string(),
                body: "Body".to_string(),
                tags: vec![],
                created_at: "ignored".to_string(),
                updated_at: "ignored".to_string(),
            })
            .unwrap();
        repo_a.push().unwrap();

        let clone_dir = TempDir::new().unwrap();
        let cloned = Repository::clone(remote_dir.path().to_str().unwrap(), clone_dir.path()).unwrap();
        let repo_b = GitEntryRepository::with_existing_repo(cloned);

        let source = repo_a.get("202608241300").unwrap().unwrap();
        let mirrored = repo_b.get("202608241300").unwrap().unwrap();

        assert_eq!(source.created_at, mirrored.created_at);
        assert_eq!(source.updated_at, mirrored.updated_at);
        assert!(
            chrono::DateTime::parse_from_rfc3339(&mirrored.created_at).is_ok(),
            "cloned entry must carry history-derived timestamp"
        );
    }

    #[test]
    fn test_sync_returns_no_remote_when_origin_missing() {
        let (_tmp_dir, repo) = create_test_repo();

        let result = repo.sync().unwrap();

        assert_eq!(result, SyncResult::NoRemote);
    }

    #[test]
    fn test_sync_pushes_to_local_bare_remote() {
        let remote_dir = TempDir::new().unwrap();
        Repository::init_bare(remote_dir.path()).unwrap();

        let (_tmp_dir, repo) = create_test_repo();
        add_origin_remote(&repo, remote_dir.path());

        repo.save(&Entry {
            id: EntryId("202608091500".to_string()),
            title: "Push Me".to_string(),
            body: "Body".to_string(),
            tags: vec![],
            created_at: "2026-08-09T15:00:00+00:00".to_string(),
            updated_at: "2026-08-09T15:00:00+00:00".to_string(),
        })
        .unwrap();

        let result = repo.sync().unwrap();

        let branch = match result {
            SyncResult::Pushed { branch } => branch,
            other => panic!("expected pushed sync result, got {:?}", other),
        };

        let remote_repo = Repository::open_bare(remote_dir.path()).unwrap();
        let remote_ref = format!("refs/heads/{}", branch);
        assert!(remote_repo.find_reference(&remote_ref).is_ok());
    }

    #[test]
    fn test_sync_fast_forwards_local_clone_from_remote() {
        let remote_dir = TempDir::new().unwrap();
        Repository::init_bare(remote_dir.path()).unwrap();

        let (_tmp_dir_a, repo_a) = create_test_repo();
        add_origin_remote(&repo_a, remote_dir.path());

        repo_a
            .save(&Entry {
                id: EntryId("202608091510".to_string()),
                title: "Base".to_string(),
                body: "Base body".to_string(),
                tags: vec![],
                created_at: "2026-08-09T15:10:00+00:00".to_string(),
                updated_at: "2026-08-09T15:10:00+00:00".to_string(),
            })
            .unwrap();
        repo_a.sync().unwrap();

        let clone_dir = TempDir::new().unwrap();
        let cloned_repo = Repository::clone(remote_dir.path().to_str().unwrap(), clone_dir.path()).unwrap();
        let repo_b = GitEntryRepository::with_existing_repo(cloned_repo);

        repo_a
            .save(&Entry {
                id: EntryId("202608091511".to_string()),
                title: "Second".to_string(),
                body: "Second body".to_string(),
                tags: vec![],
                created_at: "2026-08-09T15:11:00+00:00".to_string(),
                updated_at: "2026-08-09T15:11:00+00:00".to_string(),
            })
            .unwrap();
        repo_a.sync().unwrap();

        let result = repo_b.sync().unwrap();
        match result {
            SyncResult::Pulled { .. } => {}
            other => panic!("expected pulled sync result, got {:?}", other),
        }

        let pulled = repo_b.get("202608091511").unwrap();
        assert!(pulled.is_some());
        assert_eq!(pulled.unwrap().title, "Second");
    }

    #[test]
    fn test_clone_journal_clones_remote_repository() {
        let remote_dir = TempDir::new().unwrap();
        Repository::init_bare(remote_dir.path()).unwrap();

        let clone_parent = TempDir::new().unwrap();
        let clone_target = clone_parent.path().join("journal-clone");

        let cloner = GitJournalCloner;
        cloner
            .clone_journal(remote_dir.path().to_str().unwrap(), &clone_target)
            .unwrap();

        assert!(clone_target.join(".git").exists());
    }

    #[test]
    fn test_pull_returns_no_remote_when_origin_missing() {
        let (_tmp_dir, repo) = create_test_repo();

        let result = repo.pull().unwrap();

        assert_eq!(result, SyncResult::NoRemote);
    }

    #[test]
    fn test_push_returns_no_remote_when_origin_missing() {
        let (_tmp_dir, repo) = create_test_repo();

        let result = repo.push().unwrap();

        assert_eq!(result, SyncResult::NoRemote);
    }

    #[test]
    fn test_push_pushes_local_commits() {
        let remote_dir = TempDir::new().unwrap();
        Repository::init_bare(remote_dir.path()).unwrap();

        let (_tmp_dir, repo) = create_test_repo();
        add_origin_remote(&repo, remote_dir.path());

        repo.save(&Entry {
            id: EntryId("202608131601".to_string()),
            title: "Push target".to_string(),
            body: "Body".to_string(),
            tags: vec![],
            created_at: "2026-08-13T16:01:00+00:00".to_string(),
            updated_at: "2026-08-13T16:01:00+00:00".to_string(),
        })
        .unwrap();

        let result = repo.push().unwrap();
        assert!(matches!(result, SyncResult::Pushed { .. }));
    }

    #[test]
    fn test_pull_fast_forwards_when_remote_ahead() {
        let remote_dir = TempDir::new().unwrap();
        Repository::init_bare(remote_dir.path()).unwrap();

        let (_tmp_dir_a, repo_a) = create_test_repo();
        add_origin_remote(&repo_a, remote_dir.path());

        repo_a
            .save(&Entry {
                id: EntryId("202608131602".to_string()),
                title: "Base".to_string(),
                body: "Body".to_string(),
                tags: vec![],
                created_at: "2026-08-13T16:02:00+00:00".to_string(),
                updated_at: "2026-08-13T16:02:00+00:00".to_string(),
            })
            .unwrap();
        repo_a.push().unwrap();

        let clone_dir = TempDir::new().unwrap();
        let cloned_repo = Repository::clone(remote_dir.path().to_str().unwrap(), clone_dir.path()).unwrap();
        let repo_b = GitEntryRepository::with_existing_repo(cloned_repo);

        repo_a
            .save(&Entry {
                id: EntryId("202608131603".to_string()),
                title: "New remote".to_string(),
                body: "Body".to_string(),
                tags: vec![],
                created_at: "2026-08-13T16:03:00+00:00".to_string(),
                updated_at: "2026-08-13T16:03:00+00:00".to_string(),
            })
            .unwrap();
        repo_a.push().unwrap();

        let result = repo_b.pull().unwrap();
        assert!(matches!(result, SyncResult::Pulled { .. }));
    }

    #[test]
    fn test_resolve_path_returns_canonical_path() {
        let (tmp_dir, repo) = create_test_repo();
        let resolved = repo.resolve_path().unwrap();

        assert_eq!(resolved, tmp_dir.path().canonicalize().unwrap());
    }

    #[test]
    fn test_tag_catalog_add_list_update_remove() {
        let (_tmp_dir, repo) = create_test_repo();

        let added = repo.add_tag("work").unwrap();
        assert_eq!(added, vec!["work".to_string()]);

        let added = repo.add_tag("daily").unwrap();
        assert_eq!(added, vec!["daily".to_string(), "work".to_string()]);

        let renamed = repo.update_tag("daily", "journal").unwrap();
        assert_eq!(renamed, vec!["journal".to_string(), "work".to_string()]);

        let removed = repo.remove_tag("work").unwrap();
        assert_eq!(removed, vec!["journal".to_string()]);
    }

    #[test]
    fn test_tag_catalog_persists_to_penna_tags_json() {
        let (tmp_dir, repo) = create_test_repo();

        repo.add_tag("idea").unwrap();
        repo.add_tag("todo").unwrap();

        let path = tmp_dir.path().join(".penna/tags.json");
        assert!(path.exists());

        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(raw.contains("\"tags\""));

        let reopened = GitEntryRepository::new(tmp_dir.path().to_path_buf()).unwrap();
        let tags = reopened.list_tags().unwrap();
        assert_eq!(tags, vec!["idea".to_string(), "todo".to_string()]);
    }

    #[test]
    fn test_remove_and_update_tag_affect_all_notes() {
        let (_tmp_dir, repo) = create_test_repo();

        repo.save(&Entry {
            id: EntryId("202608131701".to_string()),
            title: "One".to_string(),
            body: "Body".to_string(),
            tags: vec!["work".to_string(), "daily".to_string()],
            created_at: "2026-08-13T17:01:00+00:00".to_string(),
            updated_at: "2026-08-13T17:01:00+00:00".to_string(),
        })
        .unwrap();

        repo.save(&Entry {
            id: EntryId("202608131702".to_string()),
            title: "Two".to_string(),
            body: "Body".to_string(),
            tags: vec!["work".to_string(), "idea".to_string()],
            created_at: "2026-08-13T17:02:00+00:00".to_string(),
            updated_at: "2026-08-13T17:02:00+00:00".to_string(),
        })
        .unwrap();

        repo.add_tag("work").unwrap();
        repo.add_tag("daily").unwrap();

        repo.update_tag("daily", "journal").unwrap();
        let first = repo.get("202608131701").unwrap().unwrap();
        assert!(first.tags.contains(&"journal".to_string()));
        assert!(!first.tags.contains(&"daily".to_string()));

        repo.remove_tag("work").unwrap();
        let first = repo.get("202608131701").unwrap().unwrap();
        let second = repo.get("202608131702").unwrap().unwrap();
        assert!(!first.tags.contains(&"work".to_string()));
        assert!(!second.tags.contains(&"work".to_string()));
    }

    #[test]
    fn test_sync_starts_merge_with_conflict_markers() {
        let remote_dir = TempDir::new().unwrap();
        Repository::init_bare(remote_dir.path()).unwrap();

        let (_tmp_a, repo_a) = create_test_repo();
        add_origin_remote(&repo_a, remote_dir.path());

        let seed = Entry {
            id: EntryId("202608250800".to_string()),
            title: "Shared".to_string(),
            body: "Original".to_string(),
            tags: vec![],
            created_at: "x".to_string(),
            updated_at: "x".to_string(),
        };
        repo_a.save(&seed).unwrap();
        repo_a.push().unwrap();

        let clone_dir = TempDir::new().unwrap();
        let cloned =
            Repository::clone(remote_dir.path().to_str().unwrap(), clone_dir.path()).unwrap();
        let repo_b = GitEntryRepository::with_existing_repo(cloned);

        let mut a_edit = repo_a.get("202608250800").unwrap().unwrap();
        a_edit.body = "Machine A text".to_string();
        repo_a.save(&a_edit).unwrap();

        let mut b_edit = repo_b.get("202608250800").unwrap().unwrap();
        b_edit.body = "Machine B text".to_string();
        repo_b.save(&b_edit).unwrap();
        repo_b.push().unwrap();

        assert!(matches!(repo_a.sync().unwrap(), SyncResult::Diverged { .. }));

        let work_file = _tmp_a.path().join("202608250800.md");
        let content = std::fs::read_to_string(&work_file).unwrap();
        assert!(content.contains("<<<<<<<"), "markers must appear: {}", content);
        assert!(content.contains(">>>>>>>"));
        assert!(content.contains("Machine A text"));
        assert!(content.contains("Machine B text"));

        let status = repo_a.status().unwrap();
        assert!(status.merge_in_progress);
        assert!(status
            .conflicted_paths
            .iter()
            .any(|p| p.ends_with("202608250800.md")));

        let listed = repo_a.list().unwrap();
        let conflicted_entry = listed.iter().find(|e| e.id.0 == "202608250800").unwrap();
        assert!(
            conflicted_entry.body.contains("<<<<<<<"),
            "reads must surface working-tree markers"
        );

        let conflict_view = repo_a.entry_conflict("202608250800").unwrap().unwrap();
        assert!(conflict_view.ours.contains("Machine A text"));
        assert!(conflict_view.theirs.contains("Machine B text"));

        assert_eq!(
            repo_a.list_conflicted_ids().unwrap(),
            vec!["202608250800".to_string()]
        );
    }

    #[test]
    fn test_resolved_save_stages_then_sync_concludes() {
        use penna_core::ports::ConflictView;

        let remote_dir = TempDir::new().unwrap();
        Repository::init_bare(remote_dir.path()).unwrap();

        let (_tmp_a, repo_a) = create_test_repo();
        add_origin_remote(&repo_a, remote_dir.path());

        let seed = Entry {
            id: EntryId("202608250900".to_string()),
            title: "Seed".to_string(),
            body: "Start".to_string(),
            tags: vec![],
            created_at: "x".to_string(),
            updated_at: "x".to_string(),
        };
        repo_a.save(&seed).unwrap();
        repo_a.push().unwrap();

        let clone_dir = TempDir::new().unwrap();
        let cloned =
            Repository::clone(remote_dir.path().to_str().unwrap(), clone_dir.path()).unwrap();
        let repo_b = GitEntryRepository::with_existing_repo(cloned);

        let mut a_edit = repo_a.get("202608250900").unwrap().unwrap();
        a_edit.body = "A version".to_string();
        repo_a.save(&a_edit).unwrap();

        let mut b_edit = repo_b.get("202608250900").unwrap().unwrap();
        b_edit.body = "B version".to_string();
        repo_b.save(&b_edit).unwrap();
        repo_b.push().unwrap();

        assert!(matches!(repo_a.sync().unwrap(), SyncResult::Diverged { .. }));

        let head_before = repo_a.status().unwrap().head_commit.clone().unwrap();

        // Frontend resolves by writing clean text through the normal save path.
        let mut resolved = repo_a.get("202608250900").unwrap().unwrap();
        resolved.body = "Merged by hand".to_string();
        repo_a.save(&resolved).unwrap();

        let head_after_stage = repo_a.status().unwrap().head_commit.clone().unwrap();
        assert_eq!(
            head_before, head_after_stage,
            "mid-merge saves stage without committing"
        );
        assert!(!std::fs::read_to_string(_tmp_a.path().join("202608250900.md"))
            .unwrap()
            .contains("<<<<<<<"));

        // Next sync detects the finished merge and concludes it.
        assert!(matches!(
            repo_a.sync().unwrap(),
            SyncResult::Pulled { .. }
        ));

        let head_concluded = repo_a.status().unwrap().head_commit.clone().unwrap();
        assert_ne!(head_before, head_concluded);
        assert!(!repo_a.status().unwrap().merge_in_progress);

        assert!(matches!(repo_a.push().unwrap(), SyncResult::Pushed { .. }));
        repo_b.pull().unwrap();
        let mirrored = repo_b.get("202608250900").unwrap().unwrap();
        assert_eq!(mirrored.body, "Merged by hand");
    }

    #[test]
    fn test_merge_policies_for_marker_less_conflicts() {
        let remote_dir = TempDir::new().unwrap();
        Repository::init_bare(remote_dir.path()).unwrap();

        let (_tmp_a, repo_a) = create_test_repo();
        add_origin_remote(&repo_a, remote_dir.path());

        for id in ["202608250901", "202608250902", "202608250903"] {
            repo_a
                .save(&Entry {
                    id: EntryId(id.to_string()),
                    title: format!("T{}", &id[id.len() - 1..]),
                    body: "Base".to_string(),
                    tags: vec![],
                    created_at: "x".to_string(),
                    updated_at: "x".to_string(),
                })
                .unwrap();
        }
        repo_a.push().unwrap();

        let clone_dir = TempDir::new().unwrap();
        let cloned =
            Repository::clone(remote_dir.path().to_str().unwrap(), clone_dir.path()).unwrap();
        let repo_b = GitEntryRepository::with_existing_repo(cloned);

        // A modifies 901, deletes 903.
        let mut a_edit = repo_a.get("202608250901").unwrap().unwrap();
        a_edit.body = "A modified".to_string();
        repo_a.save(&a_edit).unwrap();
        repo_a.delete("202608250903").unwrap();

        // B modifies 902, modifies 903 too.
        let mut b_edit = repo_b.get("202608250902").unwrap().unwrap();
        b_edit.body = "B modified".to_string();
        repo_b.save(&b_edit).unwrap();
        let mut b_other = repo_b.get("202608250903").unwrap().unwrap();
        b_other.body = "B kept this".to_string();
        repo_b.save(&b_other).unwrap();
        repo_b.push().unwrap();

        // Divergence resolves automatically via policies.
        assert!(matches!(repo_a.sync().unwrap(), SyncResult::Pulled { .. }));
        let status = repo_a.status().unwrap();
        assert!(!status.merge_in_progress, "{:?}", status.conflicted_paths);

        let kept = repo_a.get("202608250901").unwrap().unwrap();
        assert_eq!(kept.body, "A modified");

        let resurrected = repo_a.get("202608250903").unwrap().unwrap();
        assert_eq!(resurrected.body, "B kept this");

        let theirs_change = repo_a.get("202608250902").unwrap().unwrap();
        assert_eq!(theirs_change.body, "B modified");
    }

    #[test]
    fn test_tag_sidecar_divergence_auto_unions_without_markers() {
        let remote_dir = TempDir::new().unwrap();
        Repository::init_bare(remote_dir.path()).unwrap();

        let (_tmp_a, repo_a) = create_test_repo();
        add_origin_remote(&repo_a, remote_dir.path());

        repo_a
            .save(&Entry {
                id: EntryId("202608250910".to_string()),
                title: "T".to_string(),
                body: "Body".to_string(),
                tags: vec![],
                created_at: "x".to_string(),
                updated_at: "x".to_string(),
            })
            .unwrap();
        repo_a.add_tag("alpha").unwrap();
        repo_a.push().unwrap();

        let clone_dir = TempDir::new().unwrap();
        let cloned =
            Repository::clone(remote_dir.path().to_str().unwrap(), clone_dir.path()).unwrap();
        let repo_b = GitEntryRepository::with_existing_repo(cloned);
        repo_b.add_tag("beta").unwrap();
        repo_b.push().unwrap();

        repo_a.add_tag("gamma").unwrap();
        assert!(matches!(repo_a.sync().unwrap(), SyncResult::Pulled { .. }));

        assert!(!repo_a.status().unwrap().merge_in_progress);
        assert!(repo_a.list_conflicted_ids().unwrap().is_empty());
        let tags = repo_a.list_tags().unwrap();
        assert!(tags.contains(&"alpha".to_string()));
        assert!(tags.contains(&"beta".to_string()));
        assert!(tags.contains(&"gamma".to_string()));
    }
    #[test]
    fn test_attachments_round_trip_and_delete_cleanup() {
        use penna_core::ports::AttachmentStore;

        let remote_dir = TempDir::new().unwrap();
        Repository::init_bare(remote_dir.path()).unwrap();

        let (_tmp_a, repo_a) = create_test_repo();
        add_origin_remote(&repo_a, remote_dir.path());

        repo_a
            .save(&Entry {
                id: EntryId("202608241800".to_string()),
                title: "With photo".to_string(),
                body: "Body".to_string(),
                tags: vec!["trip".to_string()],
                created_at: "x".to_string(),
                updated_at: "x".to_string(),
            })
            .unwrap();
        repo_a.add_tag("daily").unwrap();

        let png = vec![0x89u8, b'P', b'N', b'G', 0xFF, 0x00, 0xAB];
        let meta = repo_a
            .add_attachment("202608241800", "photo.png", &png)
            .unwrap();
        assert_eq!(meta.bytes, png.len() as u64);

        let listed = repo_a.list_attachments("202608241800").unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "photo.png");

        let fetched = repo_a
            .get_attachment("202608241800", "photo.png")
            .unwrap()
            .unwrap();
        assert_eq!(fetched, png);
        assert!(repo_a.get_attachment("202608241800", "ghost.png").unwrap().is_none());

        let entry = repo_a.get("202608241800").unwrap().unwrap();
        assert_eq!(entry.tags, vec!["trip".to_string()]);
        let catalog = repo_a.list_tags().unwrap();
        assert!(catalog.contains(&"daily".to_string()));

        let manifest = std::fs::read_to_string(
            _tmp_a.path().join(".penna/202608241800.json"),
        )
        .unwrap();
        assert!(manifest.contains("attachments"));
        assert!(manifest.contains("photo.png"));

        repo_a.push().unwrap();

        let clone_dir = TempDir::new().unwrap();
        let cloned = Repository::clone(remote_dir.path().to_str().unwrap(), clone_dir.path()).unwrap();
        let repo_b = GitEntryRepository::with_existing_repo(cloned);
        let mirrored = repo_b
            .get_attachment("202608241800", "photo.png")
            .unwrap()
            .unwrap();
        assert_eq!(mirrored, png);

        repo_a.remove_attachment("202608241800", "photo.png").unwrap();
        assert!(repo_a.list_attachments("202608241800").unwrap().is_empty());
        assert!(!repo_a.attachment_dir("202608241800").exists());

        let meta2 = repo_a
            .add_attachment("202608241800", "second.bin", &[9u8; 10])
            .unwrap();
        assert_eq!(meta2.bytes, 10);

        repo_a.delete("202608241800").unwrap();
        assert!(repo_a.get("202608241800").unwrap().is_none());
        assert!(!repo_a.attachment_dir("202608241800").exists());
        assert!(repo_a.list_attachments("202608241800").unwrap().is_empty());
    }
}
