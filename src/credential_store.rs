use std::path::PathBuf;

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm, Nonce};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use rand::RngCore;
use std::sync::OnceLock;
use zeroize::Zeroize;

use crate::config;

fn ensure_key_dir(path: &std::path::Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ =
                std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700));
        }
    }
    Ok(())
}

fn save_key_file_exclusive(path: &std::path::Path, b64_key: &str) -> std::io::Result<()> {
    use std::io::Write;
    ensure_key_dir(path)?;

    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut file = opts.open(path)?;
    file.write_all(b64_key.as_bytes())?;
    file.sync_all()?;
    Ok(())
}

fn save_key_file(path: &std::path::Path, b64_key: &str) -> std::io::Result<()> {
    use std::io::Write;
    ensure_key_dir(path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true).truncate(true).mode(0o600);
        let mut file = options.open(path)?;
        file.write_all(b64_key.as_bytes())?;
        file.sync_all()?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, b64_key)?;
    }
    Ok(())
}

fn read_key_file(path: &std::path::Path) -> Option<[u8; 32]> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(path) {
            let mode = meta.permissions().mode();
            if mode & 0o077 != 0 {
                eprintln!(
                    "Warning: encryption key file {} has overly permissive mode {:04o}. \
                     Expected 0600. Run: chmod 600 {}",
                    path.display(),
                    mode & 0o777,
                    path.display()
                );
            }
        }
    }

    let b64_key = std::fs::read_to_string(path).ok()?;
    let mut decoded = STANDARD.decode(b64_key.trim()).ok()?;
    if decoded.len() == 32 {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&decoded);
        decoded.zeroize();
        Some(arr)
    } else {
        decoded.zeroize();
        None
    }
}

fn generate_random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyringBackend {
    Keyring,
    File,
}

impl KeyringBackend {
    fn from_env() -> Self {
        let raw = std::env::var("WEB3_CLI_KEYRING_BACKEND").unwrap_or_default();
        match raw.to_lowercase().as_str() {
            "file" => KeyringBackend::File,
            "keyring" | "" => KeyringBackend::Keyring,
            other => {
                eprintln!(
                    "Warning: unknown WEB3_CLI_KEYRING_BACKEND=\"{other}\", \
                     defaulting to \"keyring\". Valid values: \"keyring\", \"file\"."
                );
                KeyringBackend::Keyring
            }
        }
    }
}

fn key_file_path() -> PathBuf {
    config::config_dir().join(".encryption_key")
}

fn resolve_key(backend: KeyringBackend, key_file: &std::path::Path) -> anyhow::Result<[u8; 32]> {
    if backend == KeyringBackend::Keyring {
        let username = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown-user".to_string());

        if let Ok(entry) = keyring::Entry::new("web3-cli", &username) {
            match entry.get_password() {
                Ok(b64_key) => {
                    if let Ok(decoded) = STANDARD.decode(&b64_key) {
                        if decoded.len() == 32 {
                            let mut arr = [0u8; 32];
                            arr.copy_from_slice(&decoded);
                            let _ = save_key_file(key_file, &b64_key);
                            return Ok(arr);
                        }
                    }
                }
                Err(keyring::Error::NoEntry) => {
                    if let Some(key) = read_key_file(key_file) {
                        let _ = entry.set_password(&STANDARD.encode(key));
                        return Ok(key);
                    }

                    let key = generate_random_key();
                    let b64_key = STANDARD.encode(key);
                    let _ = entry.set_password(&b64_key);

                    match save_key_file_exclusive(key_file, &b64_key) {
                        Ok(()) => return Ok(key),
                        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                            if let Some(winner) = read_key_file(key_file) {
                                let _ = entry.set_password(&STANDARD.encode(winner));
                                return Ok(winner);
                            }
                            save_key_file(key_file, &b64_key)?;
                            return Ok(key);
                        }
                        Err(e) => return Err(e.into()),
                    }
                }
                Err(_) => {
                    // Keyring access failed, fall through to file
                }
            }
        }
    }

    // File fallback
    if let Some(key) = read_key_file(key_file) {
        return Ok(key);
    }

    // Generate new key
    let key = generate_random_key();
    let b64_key = STANDARD.encode(key);
    match save_key_file_exclusive(key_file, &b64_key) {
        Ok(()) => Ok(key),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            read_key_file(key_file).ok_or_else(|| anyhow::anyhow!("key file exists but is corrupt"))
        }
        Err(e) => Err(e.into()),
    }
}

fn get_or_create_key() -> anyhow::Result<[u8; 32]> {
    static KEY: OnceLock<[u8; 32]> = OnceLock::new();

    if let Some(key) = KEY.get() {
        return Ok(*key);
    }

    let backend = KeyringBackend::from_env();
    let key_file = key_file_path();
    let key = resolve_key(backend, &key_file)?;

    if KEY.set(key).is_ok() {
        Ok(key)
    } else {
        Ok(*KEY.get().expect("key must be initialized"))
    }
}

pub fn encrypt(plaintext: &[u8]) -> anyhow::Result<Vec<u8>> {
    let key = get_or_create_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| anyhow::anyhow!("Failed to create cipher: {e}"))?;

    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {e}"))?;

    let mut result = nonce.to_vec();
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

pub fn decrypt(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    if data.len() < 12 {
        anyhow::bail!("Encrypted data too short");
    }

    let key = get_or_create_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| anyhow::anyhow!("Failed to create cipher: {e}"))?;

    let nonce = Nonce::from_slice(&data[..12]);
    let plaintext = cipher.decrypt(nonce, &data[12..]).map_err(|_| {
        anyhow::anyhow!(
            "Decryption failed. Keystore may have been created with a different encryption key. \
             Run `web3 wallet reset` and recreate your wallet."
        )
    })?;

    Ok(plaintext)
}

pub fn credentials_path() -> PathBuf {
    config::config_dir().join("keystore.enc")
}

pub fn save_encrypted(json: &str) -> anyhow::Result<PathBuf> {
    let path = credentials_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ =
                std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700));
        }
    }

    let encrypted = encrypt(json.as_bytes())?;

    // Atomic write: temp file + fsync + rename to prevent corruption
    let tmp = path.with_extension("tmp");
    {
        use std::io::Write;
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(&encrypted)?;
        file.sync_all()?;
    }
    std::fs::rename(&tmp, &path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }

    Ok(path)
}

pub fn load_encrypted() -> anyhow::Result<String> {
    let path = credentials_path();
    let data = std::fs::read(&path)
        .map_err(|_| anyhow::anyhow!("No wallet found. Run `web3 wallet create` first."))?;
    let plaintext = decrypt(&data)?;
    Ok(String::from_utf8(plaintext)?)
}

pub fn keystore_exists() -> bool {
    credentials_path().exists()
}

pub fn delete_keystore() -> anyhow::Result<()> {
    let path = credentials_path();
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}
