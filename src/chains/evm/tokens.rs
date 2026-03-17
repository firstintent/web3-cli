use alloy::primitives::{Address, U256};
use alloy::signers::Signer as _;
use alloy::sol;
use anyhow::Context;
use anyhow::Result;

use super::provider::create_provider;
use super::EvmChain;
use crate::chains::{Balance, TxResult};
use crate::keys::evm_signer::parse_evm_signer;
use crate::keys::KeyManager;

sol! {
    #[sol(rpc)]
    interface IERC20 {
        function balanceOf(address account) external view returns (uint256);
        function decimals() external view returns (uint8);
        function symbol() external view returns (string);
        function transfer(address to, uint256 amount) external returns (bool);
    }
}

pub async fn erc20_balance(rpc_url: &str, address: &str, token: &str) -> Result<Balance> {
    let provider = create_provider(rpc_url)?;
    let token_addr: Address = token
        .parse()
        .with_context(|| format!("Invalid token address: {token}"))?;
    let owner: Address = address
        .parse()
        .with_context(|| format!("Invalid address: {address}"))?;

    let contract = IERC20::new(token_addr, &provider);

    // Run all 3 RPC calls concurrently — they're independent reads.
    // Bind builders first to satisfy alloy's borrow lifetimes.
    let balance_builder = contract.balanceOf(owner);
    let decimals_builder = contract.decimals();
    let symbol_builder = contract.symbol();

    let (balance_result, decimals_result, symbol_result) =
        tokio::join!(balance_builder.call(), decimals_builder.call(), symbol_builder.call());

    let balance = balance_result.context("Failed to fetch token balance")?;
    let decimals = decimals_result.context("Failed to fetch token decimals")?;
    let symbol = symbol_result.context("Failed to fetch token symbol")?;

    Ok(Balance {
        amount: format_token_amount(balance, decimals),
        symbol,
        decimals,
        raw: balance.to_string(),
    })
}

pub async fn erc20_transfer(
    chain: &EvmChain,
    keys: &KeyManager,
    to: &str,
    token: &str,
    amount: &str,
    dry_run: bool,
) -> Result<TxResult> {
    let key = keys
        .evm_key()
        .ok_or_else(|| anyhow::anyhow!("No EVM key available"))?;
    let signer = parse_evm_signer(key)?;
    let token_addr: Address = token
        .parse()
        .with_context(|| format!("Invalid token address: {token}"))?;
    let to_addr: Address = to
        .parse()
        .with_context(|| format!("Invalid recipient address: {to}"))?;

    let provider = create_provider(chain.rpc_url())?;
    let contract = IERC20::new(token_addr, &provider);

    let decimals = contract
        .decimals()
        .call()
        .await
        .context("Failed to fetch token decimals")?;

    let amount_raw = parse_token_amount(amount, decimals)?;

    if dry_run {
        let gas = contract
            .transfer(to_addr, amount_raw)
            .from(signer.address())
            .estimate_gas()
            .await
            .context("Failed to estimate gas")?;

        return Ok(TxResult {
            hash: "dry-run".to_string(),
            explorer_url: None,
            gas_used: Some(gas.to_string()),
            gas_price: None,
            dry_run: true,
        });
    }

    let wallet = alloy::network::EthereumWallet::from(
        signer.with_chain_id(Some(chain.evm_chain_id())),
    );
    let provider = alloy::providers::ProviderBuilder::new()
        .wallet(wallet)
        .connect_http(chain.rpc_url().parse()?);

    let contract = IERC20::new(token_addr, &provider);
    let tx = contract.transfer(to_addr, amount_raw).send().await?;
    let hash = format!("{:#x}", tx.tx_hash());

    Ok(TxResult {
        explorer_url: Some(chain.tx_explorer_url(&hash)),
        hash,
        gas_used: None,
        gas_price: None,
        dry_run: false,
    })
}

fn format_token_amount(raw: U256, decimals: u8) -> String {
    super::format_token_units(raw, decimals as usize)
}

fn parse_token_amount(amount: &str, decimals: u8) -> Result<U256> {
    super::parse_units(amount, decimals as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_token_amount_zero() {
        assert_eq!(format_token_amount(U256::ZERO, 18), "0.0");
    }

    #[test]
    fn format_token_amount_usdc() {
        // 1 USDC = 1_000_000 (6 decimals)
        assert_eq!(format_token_amount(U256::from(1_000_000), 6), "1.0");
    }

    #[test]
    fn parse_token_amount_usdc() {
        let amount = parse_token_amount("1.5", 6).unwrap();
        assert_eq!(amount, U256::from(1_500_000));
    }
}
