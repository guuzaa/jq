use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

pub struct TestFile {
    _dir: TempDir, // Keep TempDir alive while TestFile exists
    path: PathBuf,
}

impl TestFile {
    pub fn new(content: &str) -> Self {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let path = dir.path().join("test.json");
        fs::write(&path, content).expect("Failed to write test file");
        Self { _dir: dir, path }
    }

    pub fn path(&self) -> &str {
        self.path.to_str().unwrap()
    }
}
