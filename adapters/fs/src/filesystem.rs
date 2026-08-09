use penna_core::ports::{FileSystem, FileSystemError};
use std::path::PathBuf;
use std::fs;

#[derive(Clone)]
pub struct LocalFileSystem {
    pub root: PathBuf,
}

impl LocalFileSystem {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn full_path(&self, path: &PathBuf) -> PathBuf {
        self.root.join(path)
    }
}

impl FileSystem for LocalFileSystem {
    fn read(&self, path: &PathBuf) -> Result<Vec<u8>, FileSystemError> {
        fs::read(self.full_path(path))
            .map_err(|e| FileSystemError::Io(e.to_string()))
    }

    fn write(&self, path: &PathBuf, data: &[u8]) -> Result<(), FileSystemError> {
        fs::write(self.full_path(path), data)
            .map_err(|e| FileSystemError::Io(e.to_string()))
    }

    fn exists(&self, path: &PathBuf) -> bool {
        self.full_path(path).exists()
    }

    fn create_dir_all(&self, path: &PathBuf) -> Result<(), FileSystemError> {
        fs::create_dir_all(self.full_path(path))
            .map_err(|e| FileSystemError::Io(e.to_string()))
    }
}
