use anyhow::{Context, Result};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

use crate::config::{get_log_dir, Config};

/// Setup structured JSON logging with rotation
pub fn setup_logging(config: &Config) -> Result<impl Drop> {
    let log_dir = get_log_dir()?;

    // Create log directory
    std::fs::create_dir_all(&log_dir).context("Failed to create log directory")?;

    // Create rotating file appender (max 10MB, keep 5 files)
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("dtriage")
        .filename_suffix("log")
        .build(&log_dir)
        .context("Failed to create log appender")?;

    // Setup tracing subscriber
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().json().with_writer(non_blocking));

    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set global subscriber")?;

    Ok(_guard)
}
