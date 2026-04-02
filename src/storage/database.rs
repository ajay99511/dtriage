use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use tracing::info;

use crate::storage::models::{FileRecord, PendingAction};

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create new database connection
    pub async fn new(data_dir: &str) -> Result<Self> {
        let data_dir_path = Utf8PathBuf::from(data_dir);
        
        // Ensure data directory exists
        tokio::fs::create_dir_all(data_dir_path.as_std_path())
            .await
            .context("Failed to create data directory")?;

        let db_path = data_dir_path.join("triage.db");

        // Create connection pool with WAL mode
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .after_connect(|conn, _| {
                Box::pin(async move {
                    // Enable WAL mode for concurrent access
                    sqlx::query("PRAGMA journal_mode=WAL")
                        .execute(&mut *conn)
                        .await?;

                    // Set busy timeout
                    sqlx::query("PRAGMA busy_timeout=5000")
                        .execute(&mut *conn)
                        .await?;

                    // Enable synchronous writes (safe for local)
                    sqlx::query("PRAGMA synchronous=NORMAL")
                        .execute(&mut *conn)
                        .await?;

                    Ok(())
                })
            })
            .connect(db_path.as_str())
            .await
            .context("Failed to connect to SQLite")?;

        // Run migrations
        Self::run_migrations(&pool).await?;

        info!("Database initialized at {:?}", db_path);

        Ok(Self { pool })
    }

    /// Run database migrations
    async fn run_migrations(pool: &SqlitePool) -> Result<()> {
        // Create file registry table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS file_registry (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT NOT NULL UNIQUE,
                file_hash TEXT NOT NULL,
                original_name TEXT,
                categorized_name TEXT,
                category TEXT,
                destination_path TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create file_registry table")?;

        // Create index for fast lookups
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_file_hash
            ON file_registry(file_hash)
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create index")?;

        // Create pending_actions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS pending_actions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_id INTEGER NOT NULL,
                action_type TEXT NOT NULL,
                source_path TEXT NOT NULL,
                destination_path TEXT NOT NULL,
                executed INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (file_id) REFERENCES file_registry(id)
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create pending_actions table")?;

        Ok(())
    }

    /// Get database pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Insert or update a file record
    pub async fn upsert_file_record(
        &self,
        file_path: &str,
        file_hash: &str,
        original_name: &str,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO file_registry (file_path, file_hash, original_name, status)
            VALUES (?, ?, ?, 'pending')
            ON CONFLICT(file_path) DO UPDATE SET
                file_hash = excluded.file_hash,
                original_name = excluded.original_name,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(file_path)
        .bind(file_hash)
        .bind(original_name)
        .execute(self.pool())
        .await?;

        if result.rows_affected() > 0 {
            // Get the ID of the inserted/updated record
            let record = sqlx::query_as::<_, FileRecord>(
                "SELECT * FROM file_registry WHERE file_path = ?",
            )
            .bind(file_path)
            .fetch_one(self.pool())
            .await?;

            Ok(record.id)
        } else {
            Ok(result.last_insert_rowid())
        }
    }

    /// Update file record with categorization
    pub async fn update_categorization(
        &self,
        file_id: i64,
        category: &str,
        destination_path: &str,
        categorized_name: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE file_registry
            SET category = ?, destination_path = ?, categorized_name = ?,
                status = 'categorized', updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(category)
        .bind(destination_path)
        .bind(categorized_name)
        .bind(file_id)
        .execute(self.pool())
        .await?;

        Ok(())
    }

    /// Create a pending action
    pub async fn create_pending_action(
        &self,
        file_id: i64,
        action_type: &str,
        source_path: &str,
        destination_path: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO pending_actions (file_id, action_type, source_path, destination_path)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(file_id)
        .bind(action_type)
        .bind(source_path)
        .bind(destination_path)
        .execute(self.pool())
        .await?;

        Ok(())
    }

    /// Get all pending actions
    pub async fn get_pending_actions(&self) -> Result<Vec<PendingAction>, sqlx::Error> {
        sqlx::query_as::<_, PendingAction>(
            r#"
            SELECT * FROM pending_actions
            WHERE executed = 0
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(self.pool())
        .await
    }

    /// Get file record by path
    pub async fn get_file_record(&self, file_path: &str) -> Result<Option<FileRecord>, sqlx::Error> {
        sqlx::query_as::<_, FileRecord>(
            "SELECT * FROM file_registry WHERE file_path = ?",
        )
        .bind(file_path)
        .fetch_optional(self.pool())
        .await
    }

    /// Get file record by hash
    pub async fn get_file_record_by_hash(&self, file_hash: &str) -> Result<Option<FileRecord>, sqlx::Error> {
        sqlx::query_as::<_, FileRecord>(
            "SELECT * FROM file_registry WHERE file_hash = ?",
        )
        .bind(file_hash)
        .fetch_optional(self.pool())
        .await
    }

    /// Mark action as executed
    pub async fn mark_action_executed(&self, action_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE pending_actions SET executed = 1 WHERE id = ?",
        )
        .bind(action_id)
        .execute(self.pool())
        .await?;

        Ok(())
    }

    /// Get triage status summary
    pub async fn get_status(&self) -> Result<TriageStatus, sqlx::Error> {
        let pending_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM pending_actions WHERE executed = 0",
        )
        .fetch_one(self.pool())
        .await?;

        let total_files: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM file_registry")
            .fetch_one(self.pool())
            .await?;

        let categorized_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM file_registry WHERE status = 'categorized'",
        )
        .fetch_one(self.pool())
        .await?;

        Ok(TriageStatus {
            pending_actions: pending_count.0 as u32,
            total_files: total_files.0 as u32,
            categorized_files: categorized_count.0 as u32,
        })
    }

    /// Clean up processed files (remove executed actions and old records)
    pub async fn cleanup(&self) -> Result<u64, sqlx::Error> {
        // Delete executed actions older than 7 days
        let result = sqlx::query(
            r#"
            DELETE FROM pending_actions
            WHERE executed = 1 AND created_at < datetime('now', '-7 days')
            "#,
        )
        .execute(self.pool())
        .await?;

        Ok(result.rows_affected())
    }
}

#[derive(Debug)]
pub struct TriageStatus {
    pub pending_actions: u32,
    pub total_files: u32,
    pub categorized_files: u32,
}
