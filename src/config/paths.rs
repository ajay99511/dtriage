use anyhow::{Context, Result};
use camino::Utf8PathBuf;

/// Get the platform-specific configuration directory
pub fn get_config_dir() -> Result<Utf8PathBuf> {
    let path = dirs::config_dir()
        .context("Could not determine config directory")?
        .join("downloads-triage");

    path.try_into()
        .context("Invalid config directory path")
}

/// Get the platform-specific data directory
pub fn get_data_dir() -> Result<Utf8PathBuf> {
    let path = dirs::data_local_dir()
        .context("Could not determine data directory")?
        .join("downloads-triage");

    path.try_into()
        .context("Invalid data directory path")
}

/// Get the platform-specific log directory
pub fn get_log_dir() -> Result<Utf8PathBuf> {
    let path = dirs::data_local_dir()
        .context("Could not determine data directory")?
        .join("downloads-triage")
        .join("logs");

    path.try_into()
        .context("Invalid log directory path")
}
