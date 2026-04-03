use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use std::fs;

mod paths;
mod rules;

pub use paths::{get_config_dir, get_data_dir, get_log_dir};
pub use rules::CategorizationRule;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Path to Downloads folder
    pub downloads_dir: String,

    /// Path to data directory (SQLite, logs)
    pub data_dir: String,

    /// Path to config directory
    #[serde(skip)]
    pub config_dir: Utf8PathBuf,

    /// Categorization rules
    pub rules: Vec<CategorizationRule>,

    /// LLM configuration (optional)
    pub llm: Option<LlmConfig>,

    /// Enable dry-run mode by default
    pub dry_run: bool,

    /// Log level
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// API key service name (stored in OS keyring)
    pub api_key_service: String,

    /// Model to use (e.g., "gpt-4", "ollama/llama2")
    pub model: String,

    /// Optional custom API base URL for OpenAI-compatible endpoints
    pub api_base: Option<String>,

    /// Enable LLM-based naming
    pub enabled: bool,
}

impl Config {
    /// Load configuration from file or create default
    pub fn load() -> Result<Self> {
        let config_dir = get_config_dir()?;
        let config_path = config_dir.join("config.toml");

        if config_path.exists() {
            // Load existing config
            let content = fs::read_to_string(&config_path)
                .context("Failed to read config file")?;
            let mut config: Config = toml::from_str(&content)
                .context("Failed to parse config file")?;

            // Ensure paths are absolute
            config.config_dir = config_dir.clone();
            if !config.data_dir.is_empty() && !std::path::Path::new(&config.data_dir).is_absolute() {
                config.data_dir = config_dir.join(&config.data_dir).to_string();
            }

            Ok(config)
        } else {
            // Create default config
            let config = Self::default_config()?;
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        fs::create_dir_all(&self.config_dir)
            .context("Failed to create config directory")?;

        let config_path = self.config_dir.join("config.toml");
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(&config_path, content)
            .context("Failed to write config file")?;

        tracing::info!("Configuration saved to {:?}", config_path);
        Ok(())
    }

    /// Create default configuration
    fn default_config() -> Result<Self> {
        let config_dir = get_config_dir()?;
        let downloads_dir = dirs::download_dir()
            .context("Could not determine Downloads directory")?
            .to_string_lossy()
            .to_string();

        Ok(Self {
            downloads_dir,
            data_dir: config_dir.join("data").to_string(),
            config_dir,
            rules: CategorizationRule::defaults(),
            llm: None,
            dry_run: true,
            log_level: "info".to_string(),
        })
    }

    /// Create a test configuration
    #[cfg(test)]
    pub fn default_for_test(downloads_dir: &std::path::Path) -> Self {
        Self {
            downloads_dir: downloads_dir.to_string_lossy().to_string(),
            data_dir: downloads_dir.to_string_lossy().to_string(),
            config_dir: Utf8PathBuf::new(),
            rules: CategorizationRule::defaults(),
            llm: None,
            dry_run: true,
            log_level: "debug".to_string(),
        }
    }
}
