use anyhow::{Result};
use notify_debouncer_full::{new_debouncer, notify::{RecursiveMode, Watcher}};
use std::path::Path;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::config::Config;
use crate::storage::Database;
use crate::triage::TriageWorker;

pub struct Daemon {
    config: Config,
    db: Database,
}

impl Daemon {
    pub fn new(config: Config, db: Database) -> Self {
        Self { config, db }
    }

    /// Run the watcher daemon
    pub async fn run(self) -> Result<()> {
        info!("Starting watcher daemon for {:?}", self.config.downloads_dir);

        // Create channel for file events
        let (tx, mut rx) = mpsc::channel(100);

        // Spawn triage worker
        let triage_worker = TriageWorker::new(self.db.clone(), self.config.clone());
        let worker_handle = tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                if let Err(e) = triage_worker.process(event).await {
                    error!("Triage worker error: {}", e);
                }
            }
        });

        // Setup file watcher
        let watcher_config = self.config.clone();
        let watcher_handle = tokio::task::spawn_blocking(move || {
            // Create debouncer (prevents duplicate events)
            let mut debouncer = new_debouncer(
                Duration::from_secs(2),
                None,
                move |res| {
                    match res {
                        Ok(events) => {
                            for event in events {
                                // Send to triage worker
                                if let Err(e) = tx.blocking_send(event) {
                                    error!("Failed to send event: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            for err in e {
                                error!("Watch error: {:?}", err);
                            }
                        }
                    }
                },
            )
            .map_err(|e| anyhow::anyhow!("Failed to create debouncer: {}", e))?;

            // Start watching
            let watch_path = Path::new(&watcher_config.downloads_dir);
            debouncer
                .watcher()
                .watch(
                    watch_path,
                    RecursiveMode::NonRecursive,
                )
                .map_err(|e| anyhow::anyhow!("Failed to watch directory: {}", e))?;

            info!("Watcher started for {:?}", watcher_config.downloads_dir);

            // Keep watcher alive
            loop {
                std::thread::sleep(Duration::from_secs(1));
            }

            #[allow(unreachable_code)]
            Ok::<(), anyhow::Error>(())
        });

        // Wait for shutdown signal
        tokio::signal::ctrl_c().await?;
        info!("Shutting down daemon...");

        // Cleanup
        worker_handle.abort();
        watcher_handle.abort();

        info!("Daemon shut down successfully");

        Ok(())
    }
}
