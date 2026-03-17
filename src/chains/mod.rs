pub mod evm;
pub mod solana;
pub mod sui;

use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;

use crate::keys::KeyManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChainId {
    Ethereum,
    Polygon,
    Arbitrum,
    Base,
    Solana,
    Sui,
}

impl ChainId {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ethereum => "ethereum",
            Self::Polygon => "polygon",
            Self::Arbitrum => "arbitrum",
            Self::Base => "base",
            Self::Solana => "solana",
            Self::Sui => "sui",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ethereum" | "eth" => Some(Self::Ethereum),
            "polygon" | "matic" => Some(Self::Polygon),
            "arbitrum" | "arb" => Some(Self::Arbitrum),
            "base" => Some(Self::Base),
            "solana" | "sol" => Some(Self::Solana),
            "sui" => Some(Self::Sui),
            _ => None,
        }
    }

    pub fn is_evm(&self) -> bool {
        matches!(
            self,
            Self::Ethereum | Self::Polygon | Self::Arbitrum | Self::Base
        )
    }

    pub fn all() -> &'static [ChainId] {
        &[
            Self::Ethereum,
            Self::Polygon,
            Self::Arbitrum,
            Self::Base,
            Self::Solana,
            Self::Sui,
        ]
    }

    pub fn evm_chains() -> &'static [ChainId] {
        &[Self::Ethereum, Self::Polygon, Self::Arbitrum, Self::Base]
    }

    pub fn evm_chain_names() -> Vec<String> {
        Self::evm_chains().iter().map(|c| c.as_str().to_string()).collect()
    }

    pub fn native_token(&self) -> &'static str {
        match self {
            Self::Ethereum | Self::Arbitrum | Self::Base => "ETH",
            Self::Polygon => "MATIC",
            Self::Solana => "SOL",
            Self::Sui => "SUI",
        }
    }

    #[allow(dead_code)]
    pub fn native_decimals(&self) -> u8 {
        match self {
            Self::Ethereum | Self::Polygon | Self::Arbitrum | Self::Base => 18,
            Self::Solana => 9,
            Self::Sui => 9,
        }
    }
}

impl std::fmt::Display for ChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Serialize)]
pub struct Balance {
    pub amount: String,
    pub symbol: String,
    pub decimals: u8,
    pub raw: String,
}

#[derive(Debug, Serialize)]
pub struct TxResult {
    pub hash: String,
    pub explorer_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_used: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<String>,
    pub dry_run: bool,
}

#[derive(Debug, Serialize)]
pub struct SignatureResult {
    pub signature: String,
    pub address: String,
}

#[derive(Debug, Serialize)]
pub struct TxStatus {
    pub hash: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmations: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explorer_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TxSummary {
    pub hash: String,
    pub from: String,
    pub to: Option<String>,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<u64>,
    pub status: String,
}

#[async_trait]
pub trait Chain: Send + Sync {
    fn name(&self) -> &str;
    fn chain_id(&self) -> ChainId;
    fn derive_address(&self, keys: &KeyManager) -> Result<String>;
    async fn native_balance(&self, address: &str) -> Result<Balance>;
    async fn token_balance(&self, address: &str, token: &str) -> Result<Balance>;
    async fn send_native(
        &self,
        keys: &KeyManager,
        to: &str,
        amount: &str,
        dry_run: bool,
    ) -> Result<TxResult>;
    async fn send_token(
        &self,
        keys: &KeyManager,
        to: &str,
        token: &str,
        amount: &str,
        dry_run: bool,
    ) -> Result<TxResult>;
    fn sign_message(&self, keys: &KeyManager, message: &[u8]) -> Result<SignatureResult>;
    async fn tx_status(&self, hash: &str) -> Result<TxStatus>;
    async fn tx_history(&self, address: &str, limit: usize) -> Result<Vec<TxSummary>>;
}

pub fn resolve_chain(
    chain_name: &str,
    config: &crate::config::Config,
) -> Result<Box<dyn Chain>> {
    let chain_id =
        ChainId::from_str(chain_name).ok_or_else(|| anyhow::anyhow!("Unknown chain: {chain_name}"))?;

    let chain_config = config
        .chain_config(chain_name)
        .ok_or_else(|| anyhow::anyhow!("No configuration for chain: {chain_name}"))?;

    let rpc_url = chain_config
        .rpc_urls
        .first()
        .ok_or_else(|| anyhow::anyhow!("No RPC URL configured for chain: {chain_name}"))?;

    match chain_id {
        ChainId::Ethereum | ChainId::Polygon | ChainId::Arbitrum | ChainId::Base => {
            let evm_chain_id = chain_config.chain_id.unwrap_or(1);
            Ok(Box::new(evm::EvmChain::new(
                chain_id,
                rpc_url.clone(),
                evm_chain_id,
                chain_config.explorer_url.clone(),
            )))
        }
        ChainId::Solana => Err(anyhow::anyhow!("Solana support coming in Phase 3")),
        ChainId::Sui => Err(anyhow::anyhow!("Sui support coming in Phase 4")),
    }
}
