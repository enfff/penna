use git2::{Repository, Signature};
use penna_core::domain::{Entry, EntryId};
use penna_core::ports::{EntryRepository, RepositoryError};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryStatus {
    pub branch: Option<String>,
    pub head_commit: Option<String>,
    pub is_dirty: bool,
}

#[derive(Clone)]
pub struct GitEntryRepository {
    repo: Arc<Mutex<Repository>>,
    root: PathBuf,
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

        Ok(RepositoryStatus {
            branch,
            head_commit,
            is_dirty,
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

    fn read_file_from_commit(
        &self,
        commit_oid: git2::Oid,
        path: &std::path::Path,
    ) -> Result<Option<String>, RepositoryError> {
        let repo = self.repo.lock().unwrap();
        
        let commit = repo.find_commit(commit_oid)
            .map_err(|e| RepositoryError::Storage(format!("Failed to find commit: {}", e)))?;
        
        let tree = commit.tree()
            .map_err(|e| RepositoryError::Storage(format!("Failed to get tree: {}", e)))?;

        match tree.get_path(path) {
            Ok(entry) => {
                let object = entry.to_object(&repo)
                    .map_err(|e| RepositoryError::Storage(format!("Failed to get tree object: {}", e)))?;
                
                let blob = object.into_blob()
                    .map_err(|_| RepositoryError::Storage("Not a blob".to_string()))?;
                
                let content = String::from_utf8(blob.content().to_vec())
                    .map_err(|e| RepositoryError::Storage(format!("Invalid UTF-8: {}", e)))?;
                
                Ok(Some(content))
            }
            Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
            Err(e) => Err(RepositoryError::Storage(format!("Failed to get file: {}", e))),
        }
    }

    fn create_signature(&self) -> Result<Signature<'static>, RepositoryError> {
        Signature::now("Penna", "penna@example.com")
            .map_err(|e| RepositoryError::Storage(format!("Failed to create signature: {}", e)))
    }

    fn parse_entry_content(id: &str, content: &str) -> Result<Entry, RepositoryError> {
        Self::parse_entry_with_timestamps(id, content, None)
    }

    fn parse_entry_with_timestamps(
        id: &str, 
        content: &str, 
        timestamps: Option<(String, String)>
    ) -> Result<Entry, RepositoryError> {
        let lines: Vec<&str> = content.lines().collect();
        
        let (title, body_start) = if lines.first().map(|l| l.starts_with("# ")).unwrap_or(false) {
            (lines[0][2..].to_string(), 1)
        } else {
            ("Untitled".to_string(), 0)
        };
        
        let body = lines[body_start..].join("\n");
        
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
}

impl EntryRepository for GitEntryRepository {
    fn get(&self, id: &str) -> Result<Option<Entry>, RepositoryError> {
        let entry_path = self.entry_path(id);
        
        let commit_oid = match self.get_head_oid()? {
            Some(oid) => oid,
            None => return Ok(None),
        };

        let content = self.read_file_from_commit(commit_oid, &entry_path)?;
        
        match content {
            Some(content) => {
                let entry = Self::parse_entry_content(id, &content)?;
                Ok(Some(entry))
            }
            None => Ok(None),
        }
    }

    fn save(&self, entry: &Entry) -> Result<(), RepositoryError> {
        let entry_path = self.entry_path(&entry.id.0);
        let content = Self::format_entry_content(entry);
        let sig = self.create_signature()?;
        
        let repo = self.repo.lock().unwrap();
        
        let head = repo.head();
        let parent_commit_oid = match head {
            Ok(head) if head.is_branch() => {
                let commit = head.peel_to_commit()
                    .map_err(|e| RepositoryError::Storage(format!("Failed to get head commit: {}", e)))?;
                Some(commit.id())
            }
            _ => None,
        };
        
        let mut builder = match parent_commit_oid {
            Some(commit_oid) => {
                let commit = repo.find_commit(commit_oid)
                    .map_err(|e| RepositoryError::Storage(format!("Failed to find commit: {}", e)))?;
                let tree = commit.tree()
                    .map_err(|e| RepositoryError::Storage(format!("Failed to get tree: {}", e)))?;
                repo.treebuilder(Some(&tree))
                    .map_err(|e| RepositoryError::Storage(format!("Failed to create tree builder: {}", e)))?
            }
            None => {
                repo.treebuilder(None)
                    .map_err(|e| RepositoryError::Storage(format!("Failed to create tree builder: {}", e)))?
            }
        };

        let blob_oid = repo
            .blob(content.as_bytes())
            .map_err(|e| RepositoryError::Storage(format!("Failed to create blob: {}", e)))?;

        builder.insert(
            &entry_path,
            blob_oid,
            git2::FileMode::Blob.into(),
        ).map_err(|e| RepositoryError::Storage(format!("Failed to insert file: {}", e)))?;

        let tree_oid = builder.write()
            .map_err(|e| RepositoryError::Storage(format!("Failed to write tree: {}", e)))?;

        let tree = repo.find_tree(tree_oid)
            .map_err(|e| RepositoryError::Storage(format!("Failed to find tree: {}", e)))?;

        let parent_commit = parent_commit_oid
            .and_then(|oid| repo.find_commit(oid).ok());

        let parents: Vec<&git2::Commit> = {
            match parent_commit.as_ref() {
                Some(commit) => vec![commit],
                None => vec![],
            }
        };
        
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

        let head = repo.head();
        let parent_commit_oid = match head {
            Ok(head) if head.is_branch() => {
                let commit = head.peel_to_commit()
                    .map_err(|e| RepositoryError::Storage(format!("Failed to get head commit: {}", e)))?;
                Some(commit.id())
            }
            _ => return Ok(()),
        };

        let mut builder = match parent_commit_oid {
            Some(commit_oid) => {
                let commit = repo.find_commit(commit_oid)
                    .map_err(|e| RepositoryError::Storage(format!("Failed to find commit: {}", e)))?;
                let tree = commit.tree()
                    .map_err(|e| RepositoryError::Storage(format!("Failed to get tree: {}", e)))?;
                repo.treebuilder(Some(&tree))
                    .map_err(|e| RepositoryError::Storage(format!("Failed to create tree builder: {}", e)))?
            }
            None => return Ok(()),
        };

        builder.remove(&entry_path)
            .map_err(|e| RepositoryError::Storage(format!("Failed to remove file: {}", e)))?;

        let tree_oid = builder.write()
            .map_err(|e| RepositoryError::Storage(format!("Failed to write tree: {}", e)))?;

        let tree = repo.find_tree(tree_oid)
            .map_err(|e| RepositoryError::Storage(format!("Failed to find tree: {}", e)))?;

        let parent_commit = parent_commit_oid
            .and_then(|oid| repo.find_commit(oid).ok());

        let sig = self.create_signature()?;
        
        let parents: Vec<&git2::Commit> = {
            match parent_commit.as_ref() {
                Some(commit) => vec![commit],
                None => vec![],
            }
        };
        
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
        let mut entry_ids: Vec<String> = Vec::new();

        let commit_oid = match self.get_head_oid()? {
            Some(oid) => oid,
            None => return Ok(vec![]),
        };

        {
            let repo = self.repo.lock().unwrap();
            
            let commit = repo.find_commit(commit_oid)
                .map_err(|e| RepositoryError::Storage(format!("Failed to find commit: {}", e)))?;
            
            let tree = commit.tree()
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
        }

        let mut entries = Vec::new();
        for id in entry_ids {
            if let Ok(Some(entry)) = self.get(&id) {
                entries.push(entry);
            }
        }

        entries.sort_by(|a, b| {
            let a_ts = a.updated_at.parse::<u64>().unwrap_or(0);
            let b_ts = b.updated_at.parse::<u64>().unwrap_or(0);
            b_ts.cmp(&a_ts)
        });

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo() -> (TempDir, GitEntryRepository) {
        let tmp_dir = TempDir::new().unwrap();
        let repo = GitEntryRepository::new(tmp_dir.path().to_path_buf()).unwrap();
        (tmp_dir, repo)
    }

    #[test]
    fn test_create_and_get_entry() {
        let (_tmp_dir, repo) = create_test_repo();
        
        let entry = Entry {
            id: EntryId("test-1".to_string()),
            title: "Test Entry".to_string(),
            body: "Test body content".to_string(),
            tags: vec![],
            created_at: "123".to_string(),
            updated_at: "123".to_string(),
        };

        repo.save(&entry).unwrap();
        
        let retrieved = repo.get("test-1").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title, "Test Entry");
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
}
