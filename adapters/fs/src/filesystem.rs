use penna_core::ports::{FileSystem, FileSystemError};
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Clone)]
pub struct LocalFileSystem {
    pub root: PathBuf,
}

impl LocalFileSystem {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn full_path(&self, path: &Path) -> PathBuf {
        self.root.join(path)
    }
}

impl FileSystem for LocalFileSystem {
    fn read(&self, path: &Path) -> Result<Vec<u8>, FileSystemError> {
        fs::read(self.full_path(path))
            .map_err(|e| FileSystemError::Io(e.to_string()))
    }

    fn write(&self, path: &Path, data: &[u8]) -> Result<(), FileSystemError> {
        fs::write(self.full_path(path), data)
            .map_err(|e| FileSystemError::Io(e.to_string()))
    }

    fn exists(&self, path: &Path) -> bool {
        self.full_path(path).exists()
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), FileSystemError> {
        fs::create_dir_all(self.full_path(path))
            .map_err(|e| FileSystemError::Io(e.to_string()))
    }
}
