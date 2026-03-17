use anyhow::Result;
use clap::Subcommand;

use crate::output::OutputFormat;

#[derive(Subcommand)]
pub enum EvmCommand {
    /// Call a contract method (read-only)
    Call {
        /// Contract address
        contract: String,
        /// Method signature (e.g., "balanceOf(address)")
        method: String,
        /// Method arguments
        args: Vec<String>,
        /// Path to ABI file
        #[arg(long)]
        abi: Option<String>,
    },
    /// Send a transaction to a contract
    Send {
        /// Contract address
        contract: String,
        /// Method signature
        method: String,
        /// Method arguments
        args: Vec<String>,
        /// Path to ABI file
        #[arg(long)]
        abi: Option<String>,
        /// ETH value to send
        #[arg(long)]
        value: Option<String>,
        /// Simulate without sending
        #[arg(long)]
        dry_run: bool,
    },
    /// Fetch ABI for a contract
    Abi {
        /// Contract address
        contract: String,
    },
    /// ERC-20 token operations
    Token {
        #[command(subcommand)]
        cmd: EvmTokenCommand,
    },
}

#[derive(Subcommand)]
pub enum EvmTokenCommand {
    /// Check ERC-20 token balance
    Balance {
        /// Token contract address
        contract: String,
        /// Address to check
        #[arg(long)]
        address: Option<String>,
    },
    /// Send ERC-20 tokens
    Send {
        /// Token contract address
        contract: String,
        /// Recipient address
        to: String,
        /// Amount to send
        amount: String,
        /// Simulate without sending
        #[arg(long)]
        dry_run: bool,
    },
}

pub async fn execute(
    cmd: EvmCommand,
    _output: OutputFormat,
    _chain_name: &str,
    _network: &str,
) -> Result<()> {
    match cmd {
        EvmCommand::Call { .. } => {
            anyhow::bail!("EVM contract calls coming in Phase 2")
        }
        EvmCommand::Send { .. } => {
            anyhow::bail!("EVM contract sends coming in Phase 2")
        }
        EvmCommand::Abi { .. } => {
            anyhow::bail!("ABI fetching coming in Phase 2")
        }
        EvmCommand::Token { cmd: token_cmd } => match token_cmd {
            EvmTokenCommand::Balance { contract, address } => {
                crate::commands::balance::execute_token(
                    _output,
                    _chain_name,
                    _network,
                    &contract,
                    address.as_deref(),
                )
                .await
            }
            EvmTokenCommand::Send {
                contract,
                to,
                amount,
                dry_run,
            } => {
                crate::commands::send::execute(
                    _output,
                    _chain_name,
                    _network,
                    &to,
                    &amount,
                    Some(&contract),
                    dry_run,
                )
                .await
            }
        },
    }
}
