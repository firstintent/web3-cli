use alloy::signers::local::PrivateKeySigner;
use anyhow::{Context, Result};

pub fn parse_evm_signer(key: &str) -> Result<PrivateKeySigner> {
    let key = key.trim();
    let key = key.strip_prefix("0x").unwrap_or(key);
    let bytes = hex::decode(key).context("Invalid hex in private key")?;
    if bytes.len() != 32 {
        anyhow::bail!("Private key must be 32 bytes, got {}", bytes.len());
    }
    PrivateKeySigner::from_slice(&bytes).context("Invalid EVM private key")
}
