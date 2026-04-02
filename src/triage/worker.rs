use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use notify_debouncer_full::DebouncedEvent;
use tracing::{error, info, warn};

use crate::config::Config;
use crate::storage::Database;
use crate::triage::categorizer::Categorizer;
use crate::triage::hasher::compute_file_hash;

pub struct TriageWorker {
    db: Database,
    config: Config,
    categorizer: Categorizer,
}

impl TriageWorker {
    pub fn new(db: Database, config: Config) -> Self {
        let categorizer = Categorizer::new(config.rules.clone());
        Self {
            db,
            config,
            categorizer,
        }
    }

    /// Process a file event
    pub async fn process(&self, event: DebouncedEvent) -> Result<()> {
        // Get the path from the event
        let path = match event.paths.first() {
            Some(p) => p,
            None => {
                warn!("Received event without path, skipping");
                return Ok(());
            }
        };

        let path_str = match path.to_str() {
            Some(p) => p,
            None => {
                warn!("Received non-UTF8 path, skipping");
                return Ok(());
            }
        };

        let file_path = Utf8Path::new(path_str);

        // Only process regular files (not directories)
        if !file_path.is_file() {
            return Ok(());
        }

        // Skip hidden files and temporary files
        if let Some(name) = file_path.file_name() {
            if name.starts_with('.') || name.ends_with(".tmp") || name.ends_with(".part") {
                info!("Skipping hidden/temporary file: {:?}", file_path);
                return Ok(());
            }
        }

        info!("Processing file: {:?}", file_path);

        // Check if file already exists in registry
        if let Ok(Some(record)) = self.db.get_file_record(path_str).await {
            info!("File already in registry: {:?}", file_path);
            
            // If already categorized and action executed, skip
            if record.status == "completed" {
                return Ok(());
            }
        }

        // Compute file hash
        let file_hash = match compute_file_hash(file_path).await {
            Ok(hash) => hash,
            Err(e) => {
                error!("Failed to compute hash for {:?}: {}", file_path, e);
                return Ok(());
            }
        };

        // Check for duplicates
        if let Ok(Some(existing)) = self.db.get_file_record_by_hash(&file_hash).await {
            warn!(
                "Duplicate file detected: {:?} (same as {:?})",
                file_path, existing.file_path
            );
            // Still process but mark as duplicate
        }

        // Get original filename
        let original_name = file_path
            .file_name()
            .unwrap_or("unknown")
            .to_string();

        // Insert/update file record
        let file_id = match self.db.upsert_file_record(path_str, &file_hash, &original_name).await {
            Ok(id) => id,
            Err(e) => {
                error!("Failed to insert file record: {}", e);
                return Ok(());
            }
        };

        // Categorize the file
        if let Some(rule) = self.categorizer.categorize(file_path) {
            info!(
                "Categorized {:?} as {} -> {}",
                file_path, rule.name, rule.destination
            );

            // Build destination path
            let dest_filename = original_name.clone();
            let dest_path = PathBuf::from(&self.config.downloads_dir)
                .join(&rule.destination)
                .join(&dest_filename);
            
            let destination_path = Utf8PathBuf::from_path_buf(dest_path)
                .map_err(|_| anyhow::anyhow!("Invalid destination path"))?;

            // Update database with categorization
            if let Err(e) = self
                .db
                .update_categorization(
                    file_id,
                    &rule.name,
                    destination_path.as_str(),
                    None,
                )
                .await
            {
                error!("Failed to update categorization: {}", e);
                return Ok(());
            }

            // Create pending action
            if let Err(e) = self
                .db
                .create_pending_action(
                    file_id,
                    "move",
                    path_str,
                    destination_path.as_str(),
                )
                .await
            {
                error!("Failed to create pending action: {}", e);
                return Ok(());
            }

            if self.config.dry_run {
                info!(
                    "[DRY-RUN] Would move: {} -> {}",
                    path_str,
                    destination_path
                );
            } else {
                info!("Pending action created for: {:?}", file_path);
            }
        } else {
            info!("No matching category for: {:?}", file_path);
        }

        Ok(())
    }
}

use std::path::PathBuf;
