use anyhow::{Context, Result};
use camino::Utf8Path;
use sha2::{Digest, Sha256};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};

/// Compute SHA-256 hash of a file
pub async fn compute_file_hash(file_path: &Utf8Path) -> Result<String> {
    let file = File::open(file_path.as_std_path())
        .await
        .context("Failed to open file for hashing")?;

    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .await
            .context("Failed to read file")?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    let hash = hasher.finalize();
    Ok(hex::encode(hash))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_compute_file_hash() {
        let temp_file = NamedTempFile::new().unwrap();
        {
            let mut file = temp_file.reopen().unwrap();
            file.write_all(b"test content").unwrap();
        }

        let path = Utf8Path::from_path(temp_file.path()).unwrap();
        let hash = compute_file_hash(path).await.unwrap();

        // SHA-256 hash should be 64 hex characters
        assert_eq!(hash.len(), 64);
    }
}
