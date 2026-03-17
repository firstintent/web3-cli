use anyhow::Result;
use clap::Subcommand;
use serde::Serialize;

use crate::chains::ChainId;
use crate::config;
use crate::output::{self, OutputFormat};

#[derive(Subcommand)]
pub enum ChainCommand {
    /// List all supported chains
    List,
    /// Show current chain and network info
    Info,
    /// Set default chain
    Set {
        /// Chain name
        chain: String,
        /// Network (mainnet, testnet)
        #[arg(long)]
        network: Option<String>,
        /// Custom RPC URL
        #[arg(long)]
        rpc: Option<String>,
    },
    /// Health check all configured chains
    Status,
}

#[derive(Serialize)]
struct ChainInfo {
    name: String,
    native_token: String,
    chain_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    chain_id: Option<u64>,
    rpc_url: String,
    explorer_url: String,
}

#[derive(Serialize)]
struct ChainStatus {
    name: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    block_number: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    latency_ms: Option<u64>,
}

pub async fn execute(cmd: ChainCommand, output: OutputFormat) -> Result<()> {
    match cmd {
        ChainCommand::List => cmd_list(output),
        ChainCommand::Info => cmd_info(output),
        ChainCommand::Set {
            chain,
            network,
            rpc,
        } => cmd_set(output, &chain, network.as_deref(), rpc.as_deref()),
        ChainCommand::Status => cmd_status(output).await,
    }
}

fn cmd_list(output: OutputFormat) -> Result<()> {
    let config = config::load_config()?;
    let chains: Vec<ChainInfo> = ChainId::all()
        .iter()
        .map(|id| {
            let name = id.as_str();
            let chain_config = config.chain_config(name);
            ChainInfo {
                name: name.to_string(),
                native_token: id.native_token().to_string(),
                chain_type: if id.is_evm() {
                    "EVM".to_string()
                } else {
                    name.to_string()
                },
                chain_id: chain_config.and_then(|c| c.chain_id),
                rpc_url: chain_config
                    .and_then(|c| c.rpc_urls.first())
                    .cloned()
                    .unwrap_or_default(),
                explorer_url: chain_config
                    .map(|c| c.explorer_url.clone())
                    .unwrap_or_default(),
            }
        })
        .collect();

    match output {
        OutputFormat::Json => output::print_json(&chains)?,
        OutputFormat::Table => {
            let default = &config.default_chain;
            for c in &chains {
                let marker = if c.name == *default { " (default)" } else { "" };
                println!(
                    "  {:<12} {:<6} {:<5} {}{}",
                    c.name, c.native_token, c.chain_type,
                    c.chain_id.map(|id| id.to_string()).unwrap_or_default(),
                    marker
                );
            }
        }
    }
    Ok(())
}

fn cmd_info(output: OutputFormat) -> Result<()> {
    let config = config::load_config()?;
    let chain_name = &config.default_chain;
    let chain_config = config
        .chain_config(chain_name)
        .ok_or_else(|| anyhow::anyhow!("Default chain not configured: {chain_name}"))?;

    let info = ChainInfo {
        name: chain_name.clone(),
        native_token: ChainId::from_str(chain_name)
            .map(|id| id.native_token().to_string())
            .unwrap_or_default(),
        chain_type: ChainId::from_str(chain_name)
            .map(|id| {
                if id.is_evm() {
                    "EVM".to_string()
                } else {
                    chain_name.to_string()
                }
            })
            .unwrap_or_default(),
        chain_id: chain_config.chain_id,
        rpc_url: chain_config
            .rpc_urls
            .first()
            .cloned()
            .unwrap_or_default(),
        explorer_url: chain_config.explorer_url.clone(),
    };

    match output {
        OutputFormat::Json => output::print_json(&info)?,
        OutputFormat::Table => {
            output::print_detail_table(vec![
                ["Chain".into(), info.name],
                ["Network".into(), config.default_network.clone()],
                ["Token".into(), info.native_token],
                ["Type".into(), info.chain_type],
                [
                    "Chain ID".into(),
                    info.chain_id
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| "N/A".into()),
                ],
                ["RPC URL".into(), info.rpc_url],
                ["Explorer".into(), info.explorer_url],
            ]);
        }
    }
    Ok(())
}

fn cmd_set(
    output: OutputFormat,
    chain: &str,
    network: Option<&str>,
    rpc: Option<&str>,
) -> Result<()> {
    let _ = ChainId::from_str(chain)
        .ok_or_else(|| anyhow::anyhow!("Unknown chain: {chain}"))?;

    let mut config = config::load_config()?;
    config.default_chain = chain.to_string();

    if let Some(net) = network {
        config.default_network = net.to_string();
    }

    if let Some(rpc_url) = rpc {
        if !rpc_url.starts_with("https://") {
            anyhow::bail!("RPC URL must use HTTPS for security");
        }
        if let Some(chain_config) = config.chains.get_mut(chain) {
            chain_config.rpc_urls = vec![rpc_url.to_string()];
        }
    }

    config::save_config(&config)?;

    #[derive(Serialize)]
    struct SetResult {
        chain: String,
        network: String,
    }

    let result = SetResult {
        chain: config.default_chain.clone(),
        network: config.default_network.clone(),
    };

    match output {
        OutputFormat::Json => output::print_json(&result)?,
        OutputFormat::Table => {
            println!(
                "Default chain set to {} ({})",
                config.default_chain, config.default_network
            );
        }
    }
    Ok(())
}

async fn cmd_status(output: OutputFormat) -> Result<()> {
    let config = config::load_config()?;
    let mut statuses = Vec::new();

    for id in ChainId::all() {
        let name = id.as_str();
        let chain_config = match config.chain_config(name) {
            Some(c) => c,
            None => continue,
        };

        let rpc_url = match chain_config.rpc_urls.first() {
            Some(url) => url,
            None => continue,
        };

        if !id.is_evm() {
            statuses.push(ChainStatus {
                name: name.to_string(),
                status: "not_implemented".to_string(),
                block_number: None,
                latency_ms: None,
            });
            continue;
        }

        let start = std::time::Instant::now();
        match crate::chains::evm::provider::create_provider(rpc_url) {
            Ok(provider) => {
                use alloy::providers::Provider;
                match provider.get_block_number().await {
                    Ok(block) => {
                        let latency = start.elapsed().as_millis() as u64;
                        statuses.push(ChainStatus {
                            name: name.to_string(),
                            status: "ok".to_string(),
                            block_number: Some(block),
                            latency_ms: Some(latency),
                        });
                    }
                    Err(e) => {
                        statuses.push(ChainStatus {
                            name: name.to_string(),
                            status: format!("error: {e}"),
                            block_number: None,
                            latency_ms: None,
                        });
                    }
                }
            }
            Err(e) => {
                statuses.push(ChainStatus {
                    name: name.to_string(),
                    status: format!("error: {e}"),
                    block_number: None,
                    latency_ms: None,
                });
            }
        }
    }

    match output {
        OutputFormat::Json => output::print_json(&statuses)?,
        OutputFormat::Table => {
            for s in &statuses {
                let block = s
                    .block_number
                    .map(|b| b.to_string())
                    .unwrap_or_else(|| "-".into());
                let latency = s
                    .latency_ms
                    .map(|l| format!("{l}ms"))
                    .unwrap_or_else(|| "-".into());
                println!(
                    "  {:<12} {:<20} block={:<12} latency={}",
                    s.name, s.status, block, latency
                );
            }
        }
    }
    Ok(())
}
