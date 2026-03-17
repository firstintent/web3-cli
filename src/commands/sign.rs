use anyhow::Result;

use crate::chains;
use crate::config;
use crate::keys::KeyManager;
use crate::output::{self, OutputFormat};

pub fn execute(
    output: OutputFormat,
    chain_name: &str,
    network: &str,
    message: &str,
) -> Result<()> {
    let config = config::load_config()?;
    let chain = chains::resolve_chain(chain_name, &config)?;
    let keys = KeyManager::load()?;

    let result = chain.sign_message(&keys, message.as_bytes())?;

    match output {
        OutputFormat::Json => output::print_json_with_chain(&result, chain_name, network)?,
        OutputFormat::Table => {
            output::print_detail_table(vec![
                ["Message".into(), message.to_string()],
                ["Address".into(), result.address],
                ["Signature".into(), result.signature],
            ]);
        }
    }
    Ok(())
}
