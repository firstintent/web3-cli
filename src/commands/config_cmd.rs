use anyhow::Result;
use clap::Subcommand;
use serde::Serialize;

use crate::config;
use crate::output::{self, OutputFormat};

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Show current configuration
    Show,
    /// Set a configuration value
    Set {
        /// Key to set (e.g., default_chain, default_network)
        key: String,
        /// Value to set
        value: String,
    },
    /// Show config file path
    Path,
}

pub fn execute(cmd: ConfigCommand, output: OutputFormat) -> Result<()> {
    match cmd {
        ConfigCommand::Show => {
            let config = config::load_config()?;
            match output {
                OutputFormat::Json => output::print_json(&config)?,
                OutputFormat::Table => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&config)?
                    );
                }
            }
        }
        ConfigCommand::Set { key, value } => {
            let mut config = config::load_config()?;
            match key.as_str() {
                "default_chain" => config.default_chain = value.clone(),
                "default_network" => config.default_network = value.clone(),
                _ => anyhow::bail!("Unknown config key: {key}. Valid keys: default_chain, default_network"),
            }
            config::save_config(&config)?;

            #[derive(Serialize)]
            struct SetResult {
                key: String,
                value: String,
            }
            match output {
                OutputFormat::Json => output::print_json(&SetResult { key, value })?,
                OutputFormat::Table => println!("Config updated."),
            }
        }
        ConfigCommand::Path => {
            let path = config::config_path();
            match output {
                OutputFormat::Json => {
                    #[derive(Serialize)]
                    struct PathResult {
                        path: String,
                    }
                    output::print_json(&PathResult {
                        path: path.display().to_string(),
                    })?;
                }
                OutputFormat::Table => println!("{}", path.display()),
            }
        }
    }
    Ok(())
}
