use std::process::ExitCode;

use clap::{Parser, Subcommand};
use web3_cli::chains;
use web3_cli::commands;
use web3_cli::config;
use web3_cli::error;
use web3_cli::output;
use web3_cli::output::OutputFormat;

#[derive(Parser)]
#[command(
    name = "web3",
    about = "Multi-chain Web3 wallet CLI for AI agents",
    version,
    after_help = "Exit codes: 0=success, 1=transaction, 2=auth/key, 3=validation, 4=network, 5=internal"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format: table or json
    #[arg(short, long, global = true, default_value = "table")]
    output: OutputFormat,

    /// Chain to use (overrides default)
    #[arg(long, global = true)]
    chain: Option<String>,

    /// Network to use (mainnet, testnet)
    #[arg(long, global = true)]
    network: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage wallet (create, import, show, export, reset)
    Wallet {
        #[command(subcommand)]
        cmd: commands::wallet::WalletCommand,
    },

    /// Check balance (native token by default)
    Balance {
        #[command(subcommand)]
        cmd: Option<commands::balance::BalanceCommand>,
        /// Address to check (defaults to wallet address)
        #[arg(long)]
        address: Option<String>,
    },

    /// Send native token or tokens
    Send {
        /// Recipient address
        to: String,
        /// Amount to send
        amount: String,
        /// Token contract address (for token transfers)
        #[arg(long)]
        token: Option<String>,
        /// Simulate without sending
        #[arg(long)]
        dry_run: bool,
    },

    /// Sign a message
    Sign {
        #[command(subcommand)]
        cmd: SignCommand,
    },

    /// Transaction operations
    Tx {
        #[command(subcommand)]
        cmd: commands::tx::TxCommand,
    },

    /// Chain management
    Chain {
        #[command(subcommand)]
        cmd: commands::chain::ChainCommand,
    },

    /// Configuration management
    Config {
        #[command(subcommand)]
        cmd: commands::config_cmd::ConfigCommand,
    },

    /// Validate an address and detect chain
    Validate {
        /// Address to validate
        address: String,
    },

    /// EVM-specific contract interactions
    Evm {
        #[command(subcommand)]
        cmd: commands::evm::EvmCommand,
    },

    /// Solana-specific operations
    Solana {
        #[command(subcommand)]
        cmd: commands::solana::SolanaCommand,
    },

    /// Sui-specific operations
    Sui {
        #[command(subcommand)]
        cmd: commands::sui::SuiCommand,
    },
}

#[derive(Subcommand)]
enum SignCommand {
    /// Sign an arbitrary message
    Message {
        /// Message to sign
        message: String,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let output_format = cli.output;

    if let Err(e) = run(cli).await {
        let web3_err = categorize_error(&e);
        let exit_code = web3_err.exit_code();
        output::print_error(&web3_err, output_format);
        ExitCode::from(exit_code)
    } else {
        ExitCode::SUCCESS
    }
}

fn categorize_error(e: &anyhow::Error) -> error::Web3CliError {
    let msg = e.to_string();

    if msg.contains("No wallet found")
        || msg.contains("No EVM key")
        || msg.contains("WEB3_PRIVATE_KEY")
        || msg.contains("Decryption failed")
    {
        error::Web3CliError::Auth(msg)
    } else if msg.contains("Invalid") || msg.contains("Unknown chain") || msg.contains("bad address")
    {
        error::Web3CliError::Validation(msg)
    } else if msg.contains("Failed to fetch")
        || msg.contains("RPC")
        || msg.contains("connection")
        || msg.contains("timeout")
    {
        error::Web3CliError::Network(msg)
    } else if msg.contains("INSUFFICIENT_FUNDS")
        || msg.contains("insufficient funds")
        || msg.contains("nonce")
        || msg.contains("gas")
    {
        error::Web3CliError::Transaction { message: msg }
    } else {
        error::Web3CliError::Internal(anyhow::anyhow!("{msg}"))
    }
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    let config = config::load_config()?;
    let chain_name = cli
        .chain
        .as_deref()
        .unwrap_or(&config.default_chain)
        .to_string();
    let network = cli
        .network
        .as_deref()
        .unwrap_or(&config.default_network)
        .to_string();

    match cli.command {
        Commands::Wallet { cmd } => commands::wallet::execute(cmd, cli.output),

        Commands::Balance { cmd, address } => match cmd {
            Some(commands::balance::BalanceCommand::Token { contract, address: token_addr }) => {
                let addr = token_addr.as_deref().or(address.as_deref());
                commands::balance::execute_token(
                    cli.output,
                    &chain_name,
                    &network,
                    &contract,
                    addr,
                )
                .await
            }
            Some(commands::balance::BalanceCommand::All { address: all_addr }) => {
                let addr = all_addr.as_deref().or(address.as_deref());
                commands::balance::execute_all(cli.output, &chain_name, &network, addr).await
            }
            None => {
                commands::balance::execute_native(
                    cli.output,
                    &chain_name,
                    &network,
                    address.as_deref(),
                )
                .await
            }
        },

        Commands::Send {
            to,
            amount,
            token,
            dry_run,
        } => {
            commands::send::execute(
                cli.output,
                &chain_name,
                &network,
                &to,
                &amount,
                token.as_deref(),
                dry_run,
            )
            .await
        }

        Commands::Sign { cmd } => match cmd {
            SignCommand::Message { message } => {
                commands::sign::execute(cli.output, &chain_name, &network, &message)
            }
        },

        Commands::Tx { cmd } => {
            commands::tx::execute(cmd, cli.output, &chain_name, &network).await
        }

        Commands::Chain { cmd } => commands::chain::execute(cmd, cli.output).await,

        Commands::Config { cmd } => commands::config_cmd::execute(cmd, cli.output),

        Commands::Validate { address } => validate_address(&address, cli.output),

        Commands::Evm { cmd } => {
            commands::evm::execute(cmd, cli.output, &chain_name, &network).await
        }

        Commands::Solana { cmd } => commands::solana::execute(cmd, cli.output).await,

        Commands::Sui { cmd } => commands::sui::execute(cmd, cli.output).await,
    }
}

fn validate_address(address: &str, output: OutputFormat) -> anyhow::Result<()> {
    use serde::Serialize;

    #[derive(Serialize)]
    struct ValidationResult {
        address: String,
        valid: bool,
        chains: Vec<String>,
    }

    let mut chains = Vec::new();

    // EVM address: 0x + 40 hex chars
    if address.starts_with("0x") && address.len() == 42 {
        if hex::decode(&address[2..]).is_ok() {
            chains.extend(
                ["ethereum", "polygon", "arbitrum", "base"]
                    .iter()
                    .map(|s| s.to_string()),
            );
        }
    }

    // Solana address: base58, 32-44 chars
    if address.len() >= 32
        && address.len() <= 44
        && address.chars().all(|c| {
            c.is_ascii_alphanumeric() && c != '0' && c != 'O' && c != 'I' && c != 'l'
        })
        && !address.starts_with("0x")
    {
        chains.push("solana".into());
    }

    // Sui address: 0x + 64 hex chars
    if address.starts_with("0x") && address.len() == 66 {
        if hex::decode(&address[2..]).is_ok() {
            chains.push("sui".into());
        }
    }

    let result = ValidationResult {
        address: address.to_string(),
        valid: !chains.is_empty(),
        chains: chains.clone(),
    };

    match output {
        OutputFormat::Json => output::print_json(&result)?,
        OutputFormat::Table => {
            if chains.is_empty() {
                println!("Invalid address: {address}");
                println!("Could not detect chain from address format.");
            } else {
                output::print_detail_table(vec![
                    ["Address".into(), address.to_string()],
                    ["Valid".into(), "true".into()],
                    ["Chains".into(), chains.join(", ")],
                ]);
            }
        }
    }
    Ok(())
}
