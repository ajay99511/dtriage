// Common test utilities

use std::path::PathBuf;
use tempfile::TempDir;

/// Create a temporary directory for testing
pub fn create_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

/// Create a test file with content in a directory
pub async fn create_test_file(dir: &PathBuf, name: &str, content: &[u8]) -> PathBuf {
    let path = dir.join(name);
    tokio::fs::write(&path, content)
        .await
        .expect("Failed to create test file");
    path
}
