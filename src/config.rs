use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub rpc_urls: Vec<String>,
    pub explorer_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub default_chain: String,
    pub default_network: String,
    pub chains: HashMap<String, ChainConfig>,
}

impl Default for Config {
    fn default() -> Self {
        let mut chains = HashMap::new();

        // Public RPC endpoints sourced from chainlist.org/rpcs.json
        // Multiple URLs per chain for failover (first URL is primary)
        chains.insert(
            "ethereum".to_string(),
            ChainConfig {
                rpc_urls: vec![
                    "https://eth.drpc.org".to_string(),
                    "https://eth.llamarpc.com".to_string(),
                    "https://ethereum-rpc.publicnode.com".to_string(),
                ],
                explorer_url: "https://etherscan.io".to_string(),
                chain_id: Some(1),
            },
        );

        chains.insert(
            "polygon".to_string(),
            ChainConfig {
                rpc_urls: vec![
                    "https://polygon.drpc.org".to_string(),
                    "https://polygon-bor-rpc.publicnode.com".to_string(),
                    "https://1rpc.io/matic".to_string(),
                ],
                explorer_url: "https://polygonscan.com".to_string(),
                chain_id: Some(137),
            },
        );

        chains.insert(
            "arbitrum".to_string(),
            ChainConfig {
                rpc_urls: vec![
                    "https://arbitrum.drpc.org".to_string(),
                    "https://arb1.arbitrum.io/rpc".to_string(),
                    "https://arbitrum-one-rpc.publicnode.com".to_string(),
                ],
                explorer_url: "https://arbiscan.io".to_string(),
                chain_id: Some(42161),
            },
        );

        chains.insert(
            "base".to_string(),
            ChainConfig {
                rpc_urls: vec![
                    "https://base.drpc.org".to_string(),
                    "https://mainnet.base.org".to_string(),
                    "https://base.llamarpc.com".to_string(),
                ],
                explorer_url: "https://basescan.org".to_string(),
                chain_id: Some(8453),
            },
        );

        chains.insert(
            "solana".to_string(),
            ChainConfig {
                rpc_urls: vec!["https://api.mainnet-beta.solana.com".to_string()],
                explorer_url: "https://solscan.io".to_string(),
                chain_id: None,
            },
        );

        chains.insert(
            "sui".to_string(),
            ChainConfig {
                rpc_urls: vec!["https://fullnode.mainnet.sui.io:443".to_string()],
                explorer_url: "https://suiscan.xyz".to_string(),
                chain_id: None,
            },
        );

        Self {
            default_chain: "ethereum".to_string(),
            default_network: "mainnet".to_string(),
            chains,
        }
    }
}

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("web3-cli")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn load_config() -> Result<Config> {
    let path = config_path();
    if !path.exists() {
        let config = Config::default();
        save_config(&config)?;
        return Ok(config);
    }

    let data = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config from {}", path.display()))?;
    let config: Config = serde_json::from_str(&data)
        .with_context(|| format!("Failed to parse config from {}", path.display()))?;
    Ok(config)
}

/// Write data to a file atomically via a sibling .tmp file + rename,
/// so the target file is never left in a corrupt partial-write state.
fn atomic_write(path: &std::path::Path, data: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let tmp = path.with_extension("tmp");
    let mut file = std::fs::File::create(&tmp)?;
    file.write_all(data)?;
    file.sync_all()?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

pub fn save_config(config: &Config) -> Result<()> {
    let path = config_path();
    let dir = config_dir();

    std::fs::create_dir_all(&dir)
        .with_context(|| format!("Failed to create config directory {}", dir.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700));
    }

    let json = serde_json::to_string_pretty(config)?;
    atomic_write(&path, json.as_bytes())
        .with_context(|| format!("Failed to write config to {}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }

    Ok(())
}

impl Config {
    pub fn chain_config(&self, chain: &str) -> Option<&ChainConfig> {
        self.chains.get(chain)
    }

    pub fn rpc_url(&self, chain: &str) -> Option<&str> {
        self.chains
            .get(chain)
            .and_then(|c| c.rpc_urls.first())
            .map(|s| s.as_str())
    }

    pub fn explorer_url(&self, chain: &str) -> Option<&str> {
        self.chains.get(chain).map(|c| c.explorer_url.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_all_chains() {
        let config = Config::default();
        assert!(config.chains.contains_key("ethereum"));
        assert!(config.chains.contains_key("polygon"));
        assert!(config.chains.contains_key("arbitrum"));
        assert!(config.chains.contains_key("base"));
        assert!(config.chains.contains_key("solana"));
        assert!(config.chains.contains_key("sui"));
    }

    #[test]
    fn default_chain_is_ethereum() {
        let config = Config::default();
        assert_eq!(config.default_chain, "ethereum");
        assert_eq!(config.default_network, "mainnet");
    }

    #[test]
    fn rpc_url_returns_first() {
        let config = Config::default();
        assert_eq!(config.rpc_url("ethereum"), Some("https://eth.drpc.org"));
    }

    #[test]
    fn rpc_url_unknown_chain_returns_none() {
        let config = Config::default();
        assert_eq!(config.rpc_url("bitcoin"), None);
    }

    #[test]
    fn config_roundtrip() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.default_chain, config.default_chain);
        assert_eq!(parsed.chains.len(), config.chains.len());
    }
}
