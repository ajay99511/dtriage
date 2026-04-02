//! Integration tests for the watcher functionality

mod common;

use common::{create_temp_dir, create_test_file};
use std::time::Duration;

#[tokio::test]
async fn test_categorizer_detects_pdf() {
    use dtriage::config::{CategorizationRule, Config};
    use dtriage::storage::Database;
    use dtriage::triage::{Categorizer, TriageWorker};
    use camino::Utf8Path;

    let temp_dir = create_temp_dir();
    let downloads_dir = temp_dir.path().join("downloads");
    tokio::fs::create_dir(&downloads_dir).await.unwrap();

    // Create test PDF file
    let test_file = create_test_file(&downloads_dir, "test.pdf", b"PDF content").await;

    // Setup categorizer
    let rules = CategorizationRule::defaults();
    let categorizer = Categorizer::new(rules);

    let file_path = Utf8Path::from_path(&test_file).unwrap();
    let result = categorizer.categorize(file_path);

    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "Documents");
}

#[tokio::test]
async fn test_categorizer_detects_image() {
    use dtriage::config::CategorizationRule;
    use dtriage::triage::Categorizer;
    use camino::Utf8Path;

    let temp_dir = create_temp_dir();
    let test_file = create_test_file(temp_dir.path(), "photo.jpg", b"image data").await;

    let rules = CategorizationRule::defaults();
    let categorizer = Categorizer::new(rules);

    let file_path = Utf8Path::from_path(&test_file).unwrap();
    let result = categorizer.categorize(file_path);

    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "Images");
}

#[tokio::test]
async fn test_hasher_computes_hash() {
    use dtriage::triage::compute_file_hash;
    use camino::Utf8Path;

    let temp_dir = create_temp_dir();
    let test_file = create_test_file(temp_dir.path(), "hash_test.txt", b"test content for hashing").await;

    let file_path = Utf8Path::from_path(&test_file).unwrap();
    let hash = compute_file_hash(file_path).await.unwrap();

    // SHA-256 hash should be 64 hex characters
    assert_eq!(hash.len(), 64);

    // Same content should produce same hash
    let hash2 = compute_file_hash(file_path).await.unwrap();
    assert_eq!(hash, hash2);
}

#[tokio::test]
async fn test_database_operations() {
    use dtriage::config::get_data_dir;
    use dtriage::storage::Database;

    let temp_dir = create_temp_dir();
    let data_dir = camino::Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf()).unwrap();

    // Initialize database
    let db = Database::new(&data_dir).await.unwrap();

    // Insert a file record
    let file_id = db
        .upsert_file_record(
            "/test/file.pdf",
            "abc123hash",
            "file.pdf",
        )
        .await
        .unwrap();

    assert!(file_id > 0);

    // Retrieve the record
    let record = db.get_file_record("/test/file.pdf").await.unwrap();
    assert!(record.is_some());
    let record = record.unwrap();
    assert_eq!(record.original_name, "file.pdf");
    assert_eq!(record.file_hash, "abc123hash");

    // Update categorization
    db.update_categorization(
        file_id,
        "Documents",
        "/test/Documents/file.pdf",
        None,
    )
    .await
    .unwrap();

    // Create pending action
    db.create_pending_action(
        file_id,
        "move",
        "/test/file.pdf",
        "/test/Documents/file.pdf",
    )
    .await
    .unwrap();

    // Get pending actions
    let actions = db.get_pending_actions().await.unwrap();
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].action_type, "move");
}
