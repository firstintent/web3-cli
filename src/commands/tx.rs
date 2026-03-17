use anyhow::Result;
use clap::Subcommand;

use crate::chains;
use crate::config;
use crate::output::{self, OutputFormat};

#[derive(Subcommand)]
pub enum TxCommand {
    /// Check transaction status
    Status {
        /// Transaction hash
        hash: String,
    },
    /// View transaction history
    History {
        /// Address to check (defaults to wallet address)
        #[arg(long)]
        address: Option<String>,
        /// Maximum number of transactions
        #[arg(long, default_value = "25")]
        limit: usize,
    },
}

pub async fn execute(
    cmd: TxCommand,
    output: OutputFormat,
    chain_name: &str,
    network: &str,
) -> Result<()> {
    let config = config::load_config()?;
    let chain = chains::resolve_chain(chain_name, &config)?;

    match cmd {
        TxCommand::Status { hash } => {
            let status = chain.tx_status(&hash).await?;
            match output {
                OutputFormat::Json => {
                    output::print_json_with_chain(&status, chain_name, network)?;
                }
                OutputFormat::Table => {
                    let mut rows = vec![
                        ["Hash".into(), status.hash],
                        ["Status".into(), status.status],
                    ];
                    if let Some(block) = status.block_number {
                        rows.push(["Block".into(), block.to_string()]);
                    }
                    if let Some(conf) = status.confirmations {
                        rows.push(["Confirmations".into(), conf.to_string()]);
                    }
                    if let Some(ref url) = status.explorer_url {
                        rows.push(["Explorer".into(), url.clone()]);
                    }
                    output::print_detail_table(rows);
                }
            }
        }
        TxCommand::History { address, limit } => {
            let addr = match address {
                Some(a) => a,
                None => {
                    let keys = crate::keys::KeyManager::load()?;
                    chain.derive_address(&keys)?
                }
            };
            let history = chain.tx_history(&addr, limit).await?;
            match output {
                OutputFormat::Json => {
                    output::print_json_with_chain(&history, chain_name, network)?;
                }
                OutputFormat::Table => {
                    if history.is_empty() {
                        println!("No transactions found.");
                    } else {
                        for tx in &history {
                            println!(
                                "{} | {} -> {} | {} | {}",
                                tx.hash,
                                tx.from,
                                tx.to.as_deref().unwrap_or("contract"),
                                tx.value,
                                tx.status
                            );
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
