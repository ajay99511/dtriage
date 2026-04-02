use anyhow::Result;
use clap::Parser;
use tracing::info;

mod cli;
mod config;
mod daemon;
mod logging;
mod security;
mod storage;
mod triage;

use cli::{Cli, Commands};
use config::Config;
use daemon::Daemon;
use logging::setup_logging;
use storage::Database;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Load configuration
    let config = Config::load()?;

    // Setup logging
    let _guard = setup_logging(&config)?;

    info!("Downloads Triage Agent starting...");

    // Execute CLI command
    match cli.command {
        Commands::Daemon => {
            // Start the watcher daemon
            let db = Database::new(&config.data_dir).await?;
            let daemon = Daemon::new(config, db);
            daemon.run().await?;
        }
        Commands::Review { apply } => {
            // Review pending actions
            let db = Database::new(&config.data_dir).await?;
            cli::review::execute(db.pool(), apply).await?;
        }
        Commands::Status => {
            // Show current status
            let db = Database::new(&config.data_dir).await?;
            cli::status::execute(db.pool()).await?;
        }
        Commands::Config { subcommand } => {
            // Manage configuration
            cli::config::execute(subcommand).await?;
        }
        Commands::Clean => {
            // Clean up registry
            let db = Database::new(&config.data_dir).await?;
            cli::clean::execute(db.pool()).await?;
        }
    }

    Ok(())
}
