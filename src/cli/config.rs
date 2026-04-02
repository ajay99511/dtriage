use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Write};

use crate::config::{get_config_dir, Config};
use crate::security;

/// Config subcommand type
pub enum ConfigSubcommand {
    Show,
    Edit,
    Reset,
    SetApiKey { key: String },
    DeleteApiKey,
}

/// Execute the config command
pub async fn execute(subcommand: crate::cli::ConfigSubcommand) -> Result<()> {
    match subcommand {
        crate::cli::ConfigSubcommand::Show => show_config().await,
        crate::cli::ConfigSubcommand::Edit => edit_config(),
        crate::cli::ConfigSubcommand::Reset => reset_config(),
        crate::cli::ConfigSubcommand::SetApiKey { key } => set_api_key(key),
        crate::cli::ConfigSubcommand::DeleteApiKey => delete_api_key(),
    }
}

/// Show current configuration
async fn show_config() -> Result<()> {
    let config = Config::load()?;
    let config_path = get_config_dir()?.join("config.toml");

    println!("\n╔════════════════════════════════════════════╗");
    println!("║     Configuration                          ║");
    println!("╚════════════════════════════════════════════╝");
    println!();
    println!("  Config file: {:?}", config_path);
    println!();
    println!("  Downloads Dir:   {:?}", config.downloads_dir);
    println!("  Data Dir:        {:?}", config.data_dir);
    println!("  Dry Run:         {}", config.dry_run);
    println!("  Log Level:       {}", config.log_level);
    println!();

    if let Some(llm) = &config.llm {
        println!("  LLM Configuration:");
        println!("    Enabled:       {}", llm.enabled);
        println!("    Model:         {}", llm.model);
        println!("    API Key:       [stored in OS keyring]");
    } else {
        println!("  LLM Configuration: [not configured]");
    }
    println!();

    println!("  Categorization Rules ({}):", config.rules.len());
    println!("  ─────────────────────────────────────────");
    for rule in &config.rules {
        println!(
            "    {} (priority: {}): .{}",
            rule.name,
            rule.priority,
            rule.extensions.join(", .")
        );
    }
    println!();

    Ok(())
}

/// Edit configuration file
fn edit_config() -> Result<()> {
    let config_path = get_config_dir()?.join("config.toml");

    if !config_path.exists() {
        // Create default config first
        Config::load()?;
    }

    println!("Opening configuration file for editing...");
    println!("Config file: {:?}", config_path);
    println!();

    // Try to open with default editor based on platform
    #[cfg(target_os = "windows")]
    {
        println!("On Windows, you can edit the config file at:");
        println!("  {:?}", config_path);
        println!();
        println!("Or run: notepad {:?}", config_path);
    }

    #[cfg(not(target_os = "windows"))]
    {
        use std::process::Command;

        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
        let _ = Command::new(&editor)
            .arg(&config_path)
            .status();
    }

    Ok(())
}

/// Reset configuration to defaults
fn reset_config() -> Result<()> {
    let config_path = get_config_dir()?.join("config.toml");

    print!("Are you sure you want to reset the configuration? This will overwrite your current config. (y/N): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() == "y" {
        if config_path.exists() {
            fs::remove_file(&config_path).context("Failed to remove config file")?;
        }

        // Create new default config
        let config = Config::load()?;
        println!("Configuration reset to defaults.");
        println!("Config file: {:?}", config_path);
    } else {
        println!("Reset cancelled.");
    }

    Ok(())
}

/// Set LLM API key
pub fn set_api_key(key: String) -> Result<()> {
    let config = Config::load()?;

    let service_name = if let Some(llm) = &config.llm {
        &llm.api_key_service
    } else {
        "openai"
    };

    security::store_api_key(service_name, &key)?;
    println!("API key stored securely in OS keyring for service: {}", service_name);

    Ok(())
}

/// Delete LLM API key
fn delete_api_key() -> Result<()> {
    let config = Config::load()?;

    let service_name = if let Some(llm) = &config.llm {
        &llm.api_key_service
    } else {
        "openai"
    };

    security::delete_api_key(service_name)?;
    println!("API key deleted from OS keyring for service: {}", service_name);

    Ok(())
}
