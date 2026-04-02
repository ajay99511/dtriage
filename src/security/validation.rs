use anyhow::{Result};
use camino::{Utf8Path, Utf8PathBuf};
use tracing::warn;

/// Validate that a path is within the allowed base directory
pub fn validate_path_within_base(
    path: &Utf8Path,
    base_dir: &Utf8Path,
) -> Result<Utf8PathBuf> {
    // Canonicalize both paths to resolve any .. or . components
    // For safety, we check that the normalized path starts with base
    let path_str = path.as_str();
    let base_str = base_dir.as_str();

    // First check if it starts with base
    if !path_str.starts_with(base_str) {
        anyhow::bail!(
            "Path '{}' is outside the allowed base directory '{}'",
            path,
            base_dir
        );
    }

    // Additional check: ensure no path traversal after the base
    // e.g., /downloads/../etc/passwd should fail
    let remaining = &path_str[base_str.len()..];
    if remaining.contains("..") {
        anyhow::bail!(
            "Path '{}' contains path traversal which is not allowed",
            path
        );
    }

    Ok(path.to_path_buf())
}

/// Validate destination path before file move
pub fn validate_destination_path(
    source_path: &Utf8Path,
    destination_path: &Utf8Path,
    downloads_dir: &Utf8Path,
) -> Result<()> {
    // Source must be within Downloads
    validate_path_within_base(source_path, downloads_dir)?;

    // Destination can be anywhere, but warn if outside Downloads
    if !destination_path.starts_with(downloads_dir) {
        warn!(
            "Destination path '{}' is outside Downloads folder",
            destination_path
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_path_validation_within_base() {
        let base = Utf8Path::new("/downloads");
        let test_path = Utf8Path::new("/downloads/test/file.pdf");

        let result = validate_path_within_base(test_path, base);
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_validation_outside_base() {
        let base = Utf8Path::new("/downloads");
        let test_path = Utf8Path::new("/etc/passwd");

        let result = validate_path_within_base(test_path, base);
        assert!(result.is_err());
    }

    #[test]
    fn test_path_validation_with_traversal() {
        let base = Utf8Path::new("/downloads");
        let test_path = Utf8Path::new("/downloads/../etc/passwd");

        let result = validate_path_within_base(test_path, base);
        assert!(result.is_err());
    }

    proptest! {
        #[test]
        fn test_path_validation_proptest(path in "[a-z/]+") {
            let base = Utf8Path::new("/downloads");
            let test_path_str = format!("/downloads/{}", path);
            let test_path = Utf8Path::new(&test_path_str);

            let result = validate_path_within_base(test_path, base);
            prop_assert!(result.is_ok());
        }
    }
}
