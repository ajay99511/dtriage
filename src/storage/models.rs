use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: i64,
    pub file_path: String,
    pub file_hash: String,
    pub original_name: String,
    pub categorized_name: Option<String>,
    pub category: Option<String>,
    pub destination_path: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PendingAction {
    pub id: i64,
    pub file_id: i64,
    pub action_type: String,
    pub source_path: String,
    pub destination_path: String,
    pub executed: bool,
    pub created_at: DateTime<Utc>,
}

impl FileRecord {
    /// Check if file is a duplicate
    pub fn is_duplicate(&self, hash: &str) -> bool {
        self.file_hash == hash
    }

    /// Get full destination path
    pub fn full_destination_path(&self, base_dir: &str) -> String {
        if let Some(dest) = &self.destination_path {
            if std::path::Path::new(dest).is_absolute() {
                dest.clone()
            } else {
                std::path::Path::new(base_dir)
                    .join(dest)
                    .to_string_lossy()
                    .to_string()
            }
        } else {
            self.file_path.clone()
        }
    }
}
