use anyhow::Result;
use clap::Subcommand;
use serde::Serialize;

use crate::chains::ChainId;
use crate::credential_store;
use crate::keys::KeyManager;
use crate::output::{self, OutputFormat};

#[derive(Subcommand)]
pub enum WalletCommand {
    /// Generate a new wallet with random keys
    Create {
        /// Overwrite existing wallet
        #[arg(long)]
        force: bool,
    },
    /// Import a private key for a specific chain (reads from stdin)
    ImportKey {
        /// Chain to import the key for
        chain: String,
        /// Overwrite existing wallet
        #[arg(long)]
        force: bool,
    },
    /// Import a BIP-39 mnemonic phrase (reads from stdin)
    ImportMnemonic {
        /// Overwrite existing wallet
        #[arg(long)]
        force: bool,
    },
    /// Show wallet information
    Show,
    /// Show all derived addresses
    Addresses,
    /// Export mnemonic to stdout
    Export {
        /// Confirm you want to display the mnemonic
        #[arg(long)]
        confirm: bool,
    },
    /// Delete the encrypted keystore
    Reset {
        /// Skip confirmation
        #[arg(long)]
        force: bool,
    },
}

#[derive(Serialize)]
struct WalletInfo {
    has_wallet: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    evm_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    solana_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sui_address: Option<String>,
    has_mnemonic: bool,
    chains: Vec<String>,
}

#[derive(Serialize)]
struct WalletCreated {
    evm_address: String,
    chains: Vec<String>,
}

#[derive(Serialize)]
struct KeyImported {
    chain: String,
    address: String,
}

#[derive(Serialize)]
struct WalletExport {
    mnemonic: String,
}

fn guard_overwrite(force: bool) -> Result<()> {
    if !force && credential_store::keystore_exists() {
        anyhow::bail!(
            "A wallet already exists. Use --force to overwrite.\n\
             WARNING: This will permanently delete your existing keys."
        );
    }
    Ok(())
}

pub fn execute(cmd: WalletCommand, output: OutputFormat) -> Result<()> {
    match cmd {
        WalletCommand::Create { force } => cmd_create(output, force),
        WalletCommand::ImportKey { chain, force } => cmd_import_key(output, &chain, force),
        WalletCommand::ImportMnemonic { force } => cmd_import_mnemonic(output, force),
        WalletCommand::Show => cmd_show(output),
        WalletCommand::Addresses => cmd_addresses(output),
        WalletCommand::Export { confirm } => cmd_export(output, confirm),
        WalletCommand::Reset { force } => cmd_reset(output, force),
    }
}

fn cmd_create(output: OutputFormat, force: bool) -> Result<()> {
    guard_overwrite(force)?;

    let keys = KeyManager::create_evm()?;
    let evm_address = keys
        .evm_address()?
        .ok_or_else(|| anyhow::anyhow!("Failed to derive EVM address"))?;

    let data = WalletCreated {
        evm_address: evm_address.clone(),
        chains: ChainId::evm_chain_names(),
    };

    match output {
        OutputFormat::Json => output::print_json(&data)?,
        OutputFormat::Table => {
            output::print_detail_table(vec![
                ["EVM Address".into(), evm_address],
                ["Chains".into(), ChainId::evm_chain_names().join(", ")],
                [
                    "Status".into(),
                    "Wallet created. Keys encrypted and stored securely.".into(),
                ],
            ]);
        }
    }
    Ok(())
}

fn cmd_import_key(output: OutputFormat, chain: &str, force: bool) -> Result<()> {
    guard_overwrite(force)?;

    let chain_id = ChainId::from_str(chain)
        .ok_or_else(|| anyhow::anyhow!("Unknown chain: {chain}"))?;

    if !chain_id.is_evm() {
        anyhow::bail!("Only EVM chains supported for key import in Phase 1. Solana/Sui coming soon.");
    }

    eprintln!("Enter private key (hex, with or without 0x prefix):");
    let mut key = String::new();
    std::io::stdin().read_line(&mut key)?;
    let key = key.trim();

    if key.is_empty() {
        anyhow::bail!("No key provided");
    }

    let keys = KeyManager::import_evm_key(key)?;
    let address = keys
        .evm_address()?
        .ok_or_else(|| anyhow::anyhow!("Failed to derive address"))?;

    let data = KeyImported {
        chain: chain.to_string(),
        address: address.clone(),
    };

    match output {
        OutputFormat::Json => output::print_json(&data)?,
        OutputFormat::Table => {
            output::print_detail_table(vec![
                ["Chain".into(), chain.to_string()],
                ["Address".into(), address],
                ["Status".into(), "Key imported and encrypted.".into()],
            ]);
        }
    }
    Ok(())
}

fn cmd_import_mnemonic(_output: OutputFormat, _force: bool) -> Result<()> {
    anyhow::bail!("Mnemonic import coming in Phase 2")
}

fn cmd_show(output: OutputFormat) -> Result<()> {
    if !credential_store::keystore_exists() {
        if let Ok(_key) = std::env::var("WEB3_PRIVATE_KEY") {
            let keys = KeyManager::load()?;
            let evm_address = keys.evm_address()?;

            let data = WalletInfo {
                has_wallet: true,
                evm_address,
                solana_address: None,
                sui_address: None,
                has_mnemonic: false,
                chains: vec!["ethereum (env)".into()],
            };

            return match output {
                OutputFormat::Json => output::print_json(&data),
                OutputFormat::Table => {
                    let evm = data.evm_address.as_deref().unwrap_or("none");
                    output::print_detail_table(vec![
                        ["Source".into(), "WEB3_PRIVATE_KEY env var".into()],
                        ["EVM Address".into(), evm.to_string()],
                    ]);
                    Ok(())
                }
            };
        }
        anyhow::bail!("No wallet found. Run `web3 wallet create` or set WEB3_PRIVATE_KEY.");
    }

    let keys = KeyManager::load()?;
    let evm_address = keys.evm_address()?;

    let mut chains = Vec::new();
    if keys.evm_key().is_some() {
        chains.extend(ChainId::evm_chain_names());
    }
    if keys.solana_key().is_some() {
        chains.push("solana".into());
    }
    if keys.sui_key().is_some() {
        chains.push("sui".into());
    }

    let data = WalletInfo {
        has_wallet: true,
        evm_address,
        solana_address: None,
        sui_address: None,
        has_mnemonic: keys.mnemonic().is_some(),
        chains,
    };

    match output {
        OutputFormat::Json => output::print_json(&data)?,
        OutputFormat::Table => {
            let mut rows = vec![];
            if let Some(ref addr) = data.evm_address {
                rows.push(["EVM Address".into(), addr.clone()]);
            }
            rows.push(["Has Mnemonic".into(), data.has_mnemonic.to_string()]);
            rows.push(["Chains".into(), data.chains.join(", ")]);
            output::print_detail_table(rows);
        }
    }
    Ok(())
}

fn cmd_addresses(output: OutputFormat) -> Result<()> {
    let keys = KeyManager::load()?;

    #[derive(Serialize)]
    struct Addresses {
        #[serde(skip_serializing_if = "Option::is_none")]
        evm: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        solana: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sui: Option<String>,
    }

    let addrs = Addresses {
        evm: keys.evm_address()?,
        solana: None,
        sui: None,
    };

    match output {
        OutputFormat::Json => output::print_json(&addrs)?,
        OutputFormat::Table => {
            let mut rows = vec![];
            if let Some(ref addr) = addrs.evm {
                rows.push(["EVM (ETH/Polygon/Arb/Base)".into(), addr.clone()]);
            }
            if rows.is_empty() {
                println!("No addresses available.");
            } else {
                output::print_detail_table(rows);
            }
        }
    }
    Ok(())
}

fn cmd_export(output: OutputFormat, confirm: bool) -> Result<()> {
    if !confirm {
        anyhow::bail!("Use --confirm to export. This will display sensitive key material.");
    }

    let keys = KeyManager::load()?;

    match keys.mnemonic() {
        Some(mnemonic) => {
            let data = WalletExport {
                mnemonic: mnemonic.to_string(),
            };
            match output {
                OutputFormat::Json => output::print_json(&data)?,
                OutputFormat::Table => {
                    println!("{}", mnemonic);
                }
            }
        }
        None => {
            anyhow::bail!("No mnemonic stored. Wallet was created with a direct key import.");
        }
    }

    Ok(())
}

fn cmd_reset(output: OutputFormat, force: bool) -> Result<()> {
    if !credential_store::keystore_exists() {
        anyhow::bail!("No wallet to reset.");
    }

    if !force {
        eprintln!("This will permanently delete your encrypted keystore.");
        eprintln!("All private keys will be lost. Are you sure? [y/N]");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            anyhow::bail!("Reset cancelled.");
        }
    }

    credential_store::delete_keystore()?;

    #[derive(Serialize)]
    struct ResetResult {
        reset: bool,
    }

    match output {
        OutputFormat::Json => output::print_json(&ResetResult { reset: true })?,
        OutputFormat::Table => {
            println!("Wallet reset. Keystore deleted.");
        }
    }
    Ok(())
}
