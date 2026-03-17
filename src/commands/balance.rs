use anyhow::Result;
use clap::Subcommand;

use crate::chains;
use crate::config;
use crate::keys::KeyManager;
use crate::output::{self, OutputFormat};

#[derive(Subcommand)]
pub enum BalanceCommand {
    /// Check token balance
    Token {
        /// Token contract address or mint
        contract: String,
        /// Address to check (defaults to wallet address)
        #[arg(long)]
        address: Option<String>,
    },
    /// Check all balances (native + known tokens)
    All {
        /// Address to check (defaults to wallet address)
        #[arg(long)]
        address: Option<String>,
    },
}

pub async fn execute_native(
    output: OutputFormat,
    chain_name: &str,
    network: &str,
    address: Option<&str>,
) -> Result<()> {
    let config = config::load_config()?;
    let chain = chains::resolve_chain(chain_name, &config)?;

    let addr = match address {
        Some(a) => a.to_string(),
        None => {
            let keys = KeyManager::load()?;
            chain.derive_address(&keys)?
        }
    };

    let balance = chain.native_balance(&addr).await?;

    match output {
        OutputFormat::Json => output::print_json_with_chain(&balance, chain_name, network)?,
        OutputFormat::Table => {
            output::print_detail_table(vec![
                ["Chain".into(), chain_name.to_string()],
                ["Address".into(), addr],
                [
                    "Balance".into(),
                    format!("{} {}", balance.amount, balance.symbol),
                ],
            ]);
        }
    }
    Ok(())
}

pub async fn execute_token(
    output: OutputFormat,
    chain_name: &str,
    network: &str,
    contract: &str,
    address: Option<&str>,
) -> Result<()> {
    let config = config::load_config()?;
    let chain = chains::resolve_chain(chain_name, &config)?;

    let addr = match address {
        Some(a) => a.to_string(),
        None => {
            let keys = KeyManager::load()?;
            chain.derive_address(&keys)?
        }
    };

    let balance = chain.token_balance(&addr, contract).await?;

    match output {
        OutputFormat::Json => output::print_json_with_chain(&balance, chain_name, network)?,
        OutputFormat::Table => {
            output::print_detail_table(vec![
                ["Chain".into(), chain_name.to_string()],
                ["Token".into(), contract.to_string()],
                ["Address".into(), addr],
                [
                    "Balance".into(),
                    format!("{} {}", balance.amount, balance.symbol),
                ],
            ]);
        }
    }
    Ok(())
}

pub async fn execute_all(
    output: OutputFormat,
    chain_name: &str,
    network: &str,
    address: Option<&str>,
) -> Result<()> {
    // For Phase 1, just show native balance
    execute_native(output, chain_name, network, address).await
}
