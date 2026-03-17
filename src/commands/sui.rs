use anyhow::Result;
use clap::Subcommand;

use crate::output::OutputFormat;

#[derive(Subcommand)]
pub enum SuiCommand {
    /// Call a Move function
    MoveCall {
        /// Package ID
        package: String,
        /// module::function
        function: String,
        /// Type arguments
        #[arg(long = "type-args")]
        type_args: Vec<String>,
        /// Arguments
        #[arg(long)]
        args: Vec<String>,
        /// Simulate without sending
        #[arg(long)]
        dry_run: bool,
    },
    /// Inspect a Move function (dry-run)
    Inspect {
        /// Package ID
        package: String,
        /// module::function
        function: String,
        /// Type arguments
        #[arg(long = "type-args")]
        type_args: Vec<String>,
        /// Arguments
        #[arg(long)]
        args: Vec<String>,
    },
    /// Coin operations
    Coin {
        #[command(subcommand)]
        cmd: SuiCoinCommand,
    },
}

#[derive(Subcommand)]
pub enum SuiCoinCommand {
    /// Check coin balance
    Balance {
        /// Coin type
        #[arg(long = "type")]
        coin_type: Option<String>,
        /// Address to check
        #[arg(long)]
        address: Option<String>,
    },
    /// Send coins
    Send {
        /// Recipient address
        to: String,
        /// Amount
        amount: String,
        /// Coin type
        #[arg(long = "type")]
        coin_type: Option<String>,
        /// Simulate without sending
        #[arg(long)]
        dry_run: bool,
    },
}

pub async fn execute(
    _cmd: SuiCommand,
    _output: OutputFormat,
) -> Result<()> {
    anyhow::bail!("Sui commands coming in Phase 4")
}
