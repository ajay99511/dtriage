use clap::{Parser, Subcommand};

pub mod clean;
pub mod config;
pub mod review;
pub mod status;

#[derive(Parser)]
#[command(name = "dtriage")]
#[command(author = "Downloads Triage Team")]
#[command(version = "0.1.0")]
#[command(about = "Downloads Triage Agent - Automatically organize your Downloads folder", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the watcher daemon
    Daemon,

    /// Review pending actions
    Review {
        /// Actually execute the actions (default is dry-run)
        #[arg(long)]
        apply: bool,
    },

    /// Show current triage status
    Status,

    /// Manage configuration
    Config {
        #[command(subcommand)]
        subcommand: ConfigSubcommand,
    },

    /// Clean up processed files from registry
    Clean,
}

#[derive(Subcommand)]
pub enum ConfigSubcommand {
    /// Show current configuration
    Show,

    /// Edit configuration file
    Edit,

    /// Reset to default configuration
    Reset,

    /// Set LLM API key
    SetApiKey {
        /// API key value (stored in OS keyring)
        key: String,
    },

    /// Delete LLM API key
    DeleteApiKey,
}
