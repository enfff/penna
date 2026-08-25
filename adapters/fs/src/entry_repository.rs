use penna_core::domain::{Entry, EntryId};
use penna_core::ports::{EntryRepository, FileSystem, FileSystemError, RepositoryError};
use std::path::PathBuf;
use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct FileEntryRepository<F: FileSystem> {
    fs: Arc<F>,
    root: PathBuf,
}

impl<F: FileSystem> FileEntryRepository<F> {
    pub fn new(fs: F, root: PathBuf) -> Self {
        Self { fs: Arc::new(fs), root }
    }

    fn entry_path(&self, id: &str) -> PathBuf {
        PathBuf::from(format!("{}.md", id))
    }

    fn parse_filename(filename: &str) -> Option<String> {
        filename.strip_suffix(".md").map(str::to_string)
    }
}

impl<F: FileSystem> EntryRepository for FileEntryRepository<F> {
    fn get(&self, id: &str) -> Result<Option<Entry>, RepositoryError> {
        let path = self.entry_path(id);
        
        if !self.fs.exists(&path) {
            return Ok(None);
        }

        let content = self.fs.read(&path)
            .map_err(|e| RepositoryError::Storage(e.to_string()))?;
        
        let content = String::from_utf8_lossy(&content);
        let entry = Self::parse_entry_content(id, &content)
            .map_err(|e| RepositoryError::Storage(e.to_string()))?;
        
        Ok(Some(entry))
    }

    fn save(&self, entry: &Entry) -> Result<(), RepositoryError> {
        let path = self.entry_path(&entry.id.0);
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            let mut dir_path = self.root.clone();
            dir_path.push(parent);
            self.fs.create_dir_all(&dir_path)
                .map_err(|e| RepositoryError::Storage(e.to_string()))?;
        }

        let content = Self::format_entry_content(entry)
            .map_err(|e| RepositoryError::Storage(e.to_string()))?;
        
        self.fs.write(&path, content.as_bytes())
            .map_err(|e| RepositoryError::Storage(e.to_string()))
    }

    fn delete(&self, id: &str) -> Result<(), RepositoryError> {
        let path = self.entry_path(id);
        fs::remove_file(self.root.join(&path))
            .map_err(|e| RepositoryError::Storage(e.to_string()))
    }

    fn list(&self) -> Result<Vec<Entry>, RepositoryError> {
        let mut entries = Vec::new();
        
        let read_dir = fs::read_dir(&self.root)
            .map_err(|e| RepositoryError::Storage(e.to_string()))?;
        
        for entry in read_dir.flatten() {
            let filename = entry.file_name();
            let filename = filename.to_str().ok_or_else(|| {
                RepositoryError::Storage("Invalid filename encoding".to_string())
            })?;
            
            if let Some(id) = Self::parse_filename(filename)
                && let Ok(Some(entry)) = self.get(&id)
            {
                entries.push(entry);
            }
        }
        
        // Sort by updated_at descending (newest first)
        entries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        
        Ok(entries)
    }
}

impl<F: FileSystem> FileEntryRepository<F> {
    fn parse_entry_content(id: &str, content: &str) -> Result<Entry, FileSystemError> {
        // Simple parsing: extract title from first line, rest is body
        let lines: Vec<&str> = content.lines().collect();
        
        let (title, body_start) = if lines.first().map(|l| l.starts_with("# ")).unwrap_or(false) {
            (lines[0][2..].to_string(), 1)
        } else {
            ("Untitled".to_string(), 0)
        };
        
        let body = lines[body_start..].join("\n");
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| FileSystemError::Io(e.to_string()))?
            .as_millis()
            .to_string();

        Ok(Entry {
            id: EntryId(id.to_string()),
            title,
            body,
            tags: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    fn format_entry_content(entry: &Entry) -> Result<String, FileSystemError> {
        Ok(format!("# {}\n\n{}", entry.title, entry.body))
    }
}
