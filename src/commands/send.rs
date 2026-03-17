use anyhow::Result;
use serde::Serialize;

use crate::chains;
use crate::config;
use crate::keys::KeyManager;
use crate::output::{self, OutputFormat};

#[derive(Serialize)]
struct SendResult {
    hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    explorer_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    gas_used: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    gas_price: Option<String>,
    dry_run: bool,
}

pub async fn execute(
    output: OutputFormat,
    chain_name: &str,
    network: &str,
    to: &str,
    amount: &str,
    token: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    // Warn on zero-value sends (likely agent mistake)
    if amount == "0" || amount == "0.0" {
        eprintln!("Warning: sending zero-value transaction (will still cost gas)");
    }

    let config = config::load_config()?;
    let chain = chains::resolve_chain(chain_name, &config)?;
    let keys = KeyManager::load()?;

    let result = match token {
        Some(token_addr) => {
            chain
                .send_token(&keys, to, token_addr, amount, dry_run)
                .await?
        }
        None => chain.send_native(&keys, to, amount, dry_run).await?,
    };

    let data = SendResult {
        hash: result.hash,
        explorer_url: result.explorer_url,
        gas_used: result.gas_used,
        gas_price: result.gas_price,
        dry_run: result.dry_run,
    };

    match output {
        OutputFormat::Json => output::print_json_with_chain(&data, chain_name, network)?,
        OutputFormat::Table => {
            let mut rows = vec![
                ["To".into(), to.to_string()],
                ["Amount".into(), amount.to_string()],
            ];
            if let Some(t) = token {
                rows.push(["Token".into(), t.to_string()]);
            }
            if data.dry_run {
                rows.push(["Mode".into(), "DRY RUN (no transaction sent)".into()]);
            }
            rows.push(["Hash".into(), data.hash]);
            if let Some(ref url) = data.explorer_url {
                rows.push(["Explorer".into(), url.clone()]);
            }
            if let Some(ref gas) = data.gas_used {
                rows.push(["Gas Used".into(), gas.clone()]);
            }
            if let Some(ref price) = data.gas_price {
                rows.push(["Gas Price".into(), price.clone()]);
            }
            output::print_detail_table(rows);
        }
    }
    Ok(())
}
