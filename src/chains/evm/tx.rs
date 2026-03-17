use alloy::providers::Provider;
use alloy::signers::Signer as _;
use anyhow::{Context, Result};

use super::provider::create_provider;
use super::{parse_address, parse_ether, EvmChain};
use crate::chains::{TxResult, TxStatus};
use crate::keys::evm_signer::parse_evm_signer;
use crate::keys::KeyManager;

pub async fn send_native(
    chain: &EvmChain,
    keys: &KeyManager,
    to: &str,
    amount: &str,
    dry_run: bool,
) -> Result<TxResult> {
    let key = keys
        .evm_key()
        .ok_or_else(|| anyhow::anyhow!("No EVM key available"))?;
    let signer = parse_evm_signer(key)?;
    let to_addr = parse_address(to)?;
    let value = parse_ether(amount)?;

    if dry_run {
        let provider = create_provider(chain.rpc_url())?;

        let tx = alloy::rpc::types::TransactionRequest::default()
            .from(signer.address())
            .to(to_addr)
            .value(value);

        let gas = provider
            .estimate_gas(tx)
            .await
            .context("Failed to estimate gas")?;

        let gas_price = provider
            .get_gas_price()
            .await
            .context("Failed to fetch gas price")?;

        return Ok(TxResult {
            hash: "dry-run".to_string(),
            explorer_url: None,
            gas_used: Some(gas.to_string()),
            gas_price: Some(gas_price.to_string()),
            dry_run: true,
        });
    }

    let wallet = alloy::network::EthereumWallet::from(
        signer.with_chain_id(Some(chain.evm_chain_id())),
    );
    let provider = alloy::providers::ProviderBuilder::new()
        .wallet(wallet)
        .connect_http(chain.rpc_url().parse()?);

    let tx = alloy::rpc::types::TransactionRequest::default()
        .to(to_addr)
        .value(value);

    let pending = provider
        .send_transaction(tx)
        .await
        .context("Failed to send transaction")?;

    let hash = format!("{:#x}", pending.tx_hash());

    Ok(TxResult {
        explorer_url: Some(chain.tx_explorer_url(&hash)),
        hash,
        gas_used: None,
        gas_price: None,
        dry_run: false,
    })
}

pub async fn tx_status(chain: &EvmChain, hash: &str) -> Result<TxStatus> {
    let provider = create_provider(chain.rpc_url())?;

    let tx_hash: alloy::primitives::B256 = hash
        .parse()
        .with_context(|| format!("Invalid transaction hash: {hash}"))?;

    let receipt = provider
        .get_transaction_receipt(tx_hash)
        .await
        .context("Failed to fetch transaction receipt")?;

    match receipt {
        Some(receipt) => {
            let status = if receipt.status() {
                "confirmed"
            } else {
                "failed"
            };

            let current_block = provider.get_block_number().await.ok();
            let tx_block = receipt.block_number;
            let confirmations = match (current_block, tx_block) {
                (Some(current), Some(block)) => Some(current.saturating_sub(block)),
                _ => None,
            };

            Ok(TxStatus {
                hash: hash.to_string(),
                status: status.to_string(),
                block_number: tx_block,
                confirmations,
                explorer_url: Some(chain.tx_explorer_url(hash)),
            })
        }
        None => {
            // Check if tx is pending
            let tx = provider.get_transaction_by_hash(tx_hash).await?;
            let status = if tx.is_some() { "pending" } else { "not_found" };
            Ok(TxStatus {
                hash: hash.to_string(),
                status: status.to_string(),
                block_number: None,
                confirmations: None,
                explorer_url: Some(chain.tx_explorer_url(hash)),
            })
        }
    }
}
