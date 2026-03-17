pub mod evm_signer;
pub mod mnemonic;
pub mod solana_signer;
pub mod sui_signer;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::credential_store;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStore {
    pub version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mnemonic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evm_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solana_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sui_key: Option<String>,
}

impl Drop for KeyStore {
    fn drop(&mut self) {
        if let Some(ref mut m) = self.mnemonic {
            m.zeroize();
        }
        if let Some(ref mut k) = self.evm_key {
            k.zeroize();
        }
        if let Some(ref mut k) = self.solana_key {
            k.zeroize();
        }
        if let Some(ref mut k) = self.sui_key {
            k.zeroize();
        }
    }
}

pub struct KeyManager {
    keystore: KeyStore,
}

impl KeyManager {
    pub fn load() -> Result<Self> {
        // Check for WEB3_PRIVATE_KEY env var first
        if let Ok(key) = std::env::var("WEB3_PRIVATE_KEY") {
            // Validate the key before accepting it
            let key = key.trim().to_string();
            evm_signer::parse_evm_signer(&key)
                .context("WEB3_PRIVATE_KEY env var contains an invalid private key")?;
            let keystore = KeyStore {
                version: 1,
                mnemonic: None,
                evm_key: Some(key),
                solana_key: None,
                sui_key: None,
            };
            return Ok(Self { keystore });
        }

        let json = credential_store::load_encrypted()?;
        let keystore: KeyStore =
            serde_json::from_str(&json).context("Failed to parse keystore")?;
        Ok(Self { keystore })
    }

    pub fn from_keystore(keystore: KeyStore) -> Self {
        Self { keystore }
    }

    pub fn create_evm() -> Result<Self> {
        let signer = alloy::signers::local::PrivateKeySigner::random();
        let key_bytes = signer.to_bytes();
        let key_hex = format!("0x{}", hex::encode(key_bytes));

        let keystore = KeyStore {
            version: 1,
            mnemonic: None,
            evm_key: Some(key_hex),
            solana_key: None,
            sui_key: None,
        };

        let json = serde_json::to_string(&keystore)?;
        credential_store::save_encrypted(&json)?;

        Ok(Self { keystore })
    }

    pub fn import_evm_key(key: &str) -> Result<Self> {
        // Validate the key by trying to parse it
        let key_str = key.trim();
        let _ = evm_signer::parse_evm_signer(key_str)?;

        let keystore = KeyStore {
            version: 1,
            mnemonic: None,
            evm_key: Some(key_str.to_string()),
            solana_key: None,
            sui_key: None,
        };

        let json = serde_json::to_string(&keystore)?;
        credential_store::save_encrypted(&json)?;

        Ok(Self { keystore })
    }

    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string(&self.keystore)?;
        credential_store::save_encrypted(&json)?;
        Ok(())
    }

    pub fn evm_key(&self) -> Option<&str> {
        self.keystore.evm_key.as_deref()
    }

    pub fn solana_key(&self) -> Option<&str> {
        self.keystore.solana_key.as_deref()
    }

    pub fn sui_key(&self) -> Option<&str> {
        self.keystore.sui_key.as_deref()
    }

    pub fn mnemonic(&self) -> Option<&str> {
        self.keystore.mnemonic.as_deref()
    }

    pub fn has_key_for(&self, chain: &str) -> bool {
        match chain {
            "ethereum" | "polygon" | "arbitrum" | "base" => self.keystore.evm_key.is_some(),
            "solana" => self.keystore.solana_key.is_some(),
            "sui" => self.keystore.sui_key.is_some(),
            _ => false,
        }
    }

    pub fn evm_address(&self) -> Result<Option<String>> {
        match self.keystore.evm_key.as_deref() {
            Some(key) => {
                let signer = evm_signer::parse_evm_signer(key)?;
                Ok(Some(format!("{}", signer.address())))
            }
            None => Ok(None),
        }
    }
}
