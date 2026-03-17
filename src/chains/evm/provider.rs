use alloy::providers::ProviderBuilder;
use anyhow::{Context, Result};

pub fn create_provider(
    rpc_url: &str,
) -> Result<impl alloy::providers::Provider + Clone> {
    let url = rpc_url
        .parse()
        .with_context(|| format!("Invalid RPC URL: {rpc_url}"))?;
    let provider = ProviderBuilder::new().connect_http(url);
    Ok(provider)
}
