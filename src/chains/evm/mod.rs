pub mod contract;
pub mod provider;
pub mod tokens;
pub mod tx;

use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use anyhow::{Context, Result};
use async_trait::async_trait;

use super::{Balance, Chain, ChainId, SignatureResult, TxResult, TxStatus, TxSummary};
use crate::keys::evm_signer::parse_evm_signer;
use crate::keys::KeyManager;

pub struct EvmChain {
    chain_id: ChainId,
    rpc_url: String,
    evm_chain_id: u64,
    explorer_url: String,
}

impl EvmChain {
    pub fn new(
        chain_id: ChainId,
        rpc_url: String,
        evm_chain_id: u64,
        explorer_url: String,
    ) -> Self {
        Self {
            chain_id,
            rpc_url,
            evm_chain_id,
            explorer_url,
        }
    }

    fn tx_explorer_url(&self, hash: &str) -> String {
        format!("{}/tx/{}", self.explorer_url, hash)
    }

    #[allow(dead_code)]
    fn address_explorer_url(&self, address: &str) -> String {
        format!("{}/address/{}", self.explorer_url, address)
    }

    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }

    pub fn evm_chain_id(&self) -> u64 {
        self.evm_chain_id
    }
}

fn parse_address(s: &str) -> Result<Address> {
    s.parse::<Address>()
        .with_context(|| format!("Invalid EVM address: {s}"))
}

fn format_ether(wei: U256) -> String {
    if wei.is_zero() {
        return "0.0".to_string();
    }
    format_token_units(wei, 18)
}

pub(crate) fn format_token_units(raw: U256, decimals: usize) -> String {
    if raw.is_zero() {
        return "0.0".to_string();
    }
    let raw_str = raw.to_string();

    if raw_str.len() <= decimals {
        let zeros = decimals - raw_str.len();
        let frac = format!("{}{}", "0".repeat(zeros), raw_str);
        let trimmed = frac.trim_end_matches('0');
        if trimmed.is_empty() {
            "0.0".to_string()
        } else {
            format!("0.{trimmed}")
        }
    } else {
        let split = raw_str.len() - decimals;
        let integer = &raw_str[..split];
        let fraction = raw_str[split..].trim_end_matches('0');
        if fraction.is_empty() {
            format!("{integer}.0")
        } else {
            format!("{integer}.{fraction}")
        }
    }
}

fn parse_ether(amount: &str) -> Result<U256> {
    parse_units(amount, 18)
}

/// Parse a decimal amount string into raw units given a number of decimals.
/// e.g., parse_units("1.5", 18) → 1_500_000_000_000_000_000
pub(crate) fn parse_units(amount: &str, decimals: usize) -> Result<U256> {
    if amount.starts_with('-') {
        anyhow::bail!("Negative amounts not allowed");
    }
    let parts: Vec<&str> = amount.split('.').collect();
    match parts.len() {
        1 => {
            let raw = U256::from_str_radix(parts[0], 10)
                .context("Invalid amount")?
                .checked_mul(U256::from(10u64).pow(U256::from(decimals)))
                .context("Amount overflow")?;
            Ok(raw)
        }
        2 => {
            let integer = if parts[0].is_empty() {
                U256::ZERO
            } else {
                U256::from_str_radix(parts[0], 10).context("Invalid integer part")?
            };
            let frac_str = parts[1];
            if frac_str.len() > decimals {
                anyhow::bail!("Too many decimal places (max {decimals})");
            }
            let padded = format!("{:0<width$}", frac_str, width = decimals);
            let frac_raw = U256::from_str_radix(&padded, 10).context("Invalid fractional part")?;
            let integer_raw = integer
                .checked_mul(U256::from(10u64).pow(U256::from(decimals)))
                .context("Amount overflow")?;
            Ok(integer_raw + frac_raw)
        }
        _ => anyhow::bail!("Invalid amount format: {amount}"),
    }
}

#[async_trait]
impl Chain for EvmChain {
    fn name(&self) -> &str {
        self.chain_id.as_str()
    }

    fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    fn derive_address(&self, keys: &KeyManager) -> Result<String> {
        let key = keys
            .evm_key()
            .ok_or_else(|| anyhow::anyhow!("No EVM key available"))?;
        let signer = parse_evm_signer(key)?;
        Ok(format!("{}", signer.address()))
    }

    async fn native_balance(&self, address: &str) -> Result<Balance> {
        let addr = parse_address(address)?;
        let provider = provider::create_provider(&self.rpc_url)?;
        let balance = provider
            .get_balance(addr)
            .await
            .context("Failed to fetch balance")?;

        Ok(Balance {
            amount: format_ether(balance),
            symbol: self.chain_id.native_token().to_string(),
            decimals: 18,
            raw: balance.to_string(),
        })
    }

    async fn token_balance(&self, address: &str, token: &str) -> Result<Balance> {
        tokens::erc20_balance(&self.rpc_url, address, token).await
    }

    async fn send_native(
        &self,
        keys: &KeyManager,
        to: &str,
        amount: &str,
        dry_run: bool,
    ) -> Result<TxResult> {
        tx::send_native(self, keys, to, amount, dry_run).await
    }

    async fn send_token(
        &self,
        keys: &KeyManager,
        to: &str,
        token: &str,
        amount: &str,
        dry_run: bool,
    ) -> Result<TxResult> {
        tokens::erc20_transfer(self, keys, to, token, amount, dry_run).await
    }

    fn sign_message(&self, keys: &KeyManager, message: &[u8]) -> Result<SignatureResult> {
        use alloy::signers::SignerSync;

        let key = keys
            .evm_key()
            .ok_or_else(|| anyhow::anyhow!("No EVM key available"))?;
        let signer = parse_evm_signer(key)?;
        let address = format!("{}", signer.address());

        // EIP-191 personal sign
        let hash = alloy::primitives::eip191_hash_message(message);
        let signature = signer
            .sign_hash_sync(&hash)
            .context("Failed to sign message")?;

        Ok(SignatureResult {
            signature: format!("0x{}", hex::encode(signature.as_bytes())),
            address,
        })
    }

    async fn tx_status(&self, hash: &str) -> Result<TxStatus> {
        tx::tx_status(self, hash).await
    }

    async fn tx_history(&self, _address: &str, _limit: usize) -> Result<Vec<TxSummary>> {
        // Transaction history requires an indexer (Etherscan API, etc.)
        // For Phase 1, return empty with a note
        Err(anyhow::anyhow!(
            "Transaction history requires an indexer API (coming in Phase 2)"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_ether_zero() {
        assert_eq!(format_ether(U256::ZERO), "0.0");
    }

    #[test]
    fn format_ether_one_wei() {
        assert_eq!(format_ether(U256::from(1)), "0.000000000000000001");
    }

    #[test]
    fn format_ether_one_eth() {
        let one_eth = U256::from(10u64).pow(U256::from(18));
        assert_eq!(format_ether(one_eth), "1.0");
    }

    #[test]
    fn format_ether_fractional() {
        let amount = U256::from(1_500_000_000_000_000_000u64);
        assert_eq!(format_ether(amount), "1.5");
    }

    #[test]
    fn parse_ether_integer() {
        let wei = parse_ether("1").unwrap();
        assert_eq!(wei, U256::from(10u64).pow(U256::from(18)));
    }

    #[test]
    fn parse_ether_fractional() {
        let wei = parse_ether("1.5").unwrap();
        assert_eq!(wei, U256::from(1_500_000_000_000_000_000u64));
    }

    #[test]
    fn parse_ether_small() {
        let wei = parse_ether("0.001").unwrap();
        assert_eq!(wei, U256::from(1_000_000_000_000_000u64));
    }

    #[test]
    fn parse_address_valid() {
        let addr = parse_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
        assert!(addr.is_ok());
    }

    #[test]
    fn parse_address_invalid() {
        let addr = parse_address("not_an_address");
        assert!(addr.is_err());
    }
}
