use anyhow::{Context, Result};
use keyring::Entry;
use tracing::info;

const SERVICE_NAME: &str = "downloads-triage";

/// Store API key in OS-native keyring
pub fn store_api_key(service: &str, key: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, service).context("Failed to create keyring entry")?;

    entry
        .set_password(key)
        .context("Failed to store API key")?;

    info!("API key stored securely in OS keyring");
    Ok(())
}

/// Retrieve API key from OS-native keyring
pub fn retrieve_api_key(service: &str) -> Result<String> {
    let entry = Entry::new(SERVICE_NAME, service).context("Failed to create keyring entry")?;

    entry
        .get_password()
        .context("Failed to retrieve API key (not set or inaccessible)")
}

/// Delete API key from keyring
pub fn delete_api_key(service: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, service).context("Failed to create keyring entry")?;

    entry
        .delete_password()
        .context("Failed to delete API key")?;

    info!("API key deleted from OS keyring");
    Ok(())
}
