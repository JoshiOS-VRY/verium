//! Encrypted backup of the BIP39 phrase when HD seed is applied on a full-node wallet.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::coin_profile::CoinId;
use crate::error::{AppError, AppResult};
use crate::secret_store;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MnemonicBackupRecord {
    encrypted_mnemonic: String,
    salt: String,
    nonce: String,
    created_at: u64,
}

fn backup_path(coin: CoinId) -> PathBuf {
    crate::config::app_config_base()
        .join("secure")
        .join(format!("hd-mnemonic-{}.enc", coin.as_str()))
}

pub fn exists(coin: CoinId) -> bool {
    backup_path(coin).exists()
}

pub fn save(coin: CoinId, mnemonic: &str, wallet_passphrase: &str) -> AppResult<()> {
    if wallet_passphrase.is_empty() {
        return Err(AppError::other(
            "Wallet passphrase is required to store a recovery phrase backup.",
        ));
    }
    let (encrypted, salt, nonce) =
        secret_store::encrypt_with_passphrase(mnemonic.as_bytes(), wallet_passphrase)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let record = MnemonicBackupRecord {
        encrypted_mnemonic: hex::encode(encrypted),
        salt: hex::encode(salt),
        nonce: hex::encode(nonce),
        created_at: now,
    };
    let path = backup_path(coin);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(&record)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn load(coin: CoinId, wallet_passphrase: &str) -> AppResult<Option<String>> {
    let path = backup_path(coin);
    if !path.exists() {
        return Ok(None);
    }
    if wallet_passphrase.is_empty() {
        return Err(AppError::other(
            "Wallet passphrase is required to decrypt the recovery phrase backup.",
        ));
    }
    let json = std::fs::read_to_string(path)?;
    let record: MnemonicBackupRecord = serde_json::from_str(&json)
        .map_err(|e| AppError::other(format!("invalid mnemonic backup file: {e}")))?;
    let encrypted = hex::decode(record.encrypted_mnemonic.trim())
        .map_err(|e| AppError::other(format!("backup ciphertext: {e}")))?;
    let salt = hex::decode(record.salt.trim())
        .map_err(|e| AppError::other(format!("backup salt: {e}")))?;
    let nonce = hex::decode(record.nonce.trim())
        .map_err(|e| AppError::other(format!("backup nonce: {e}")))?;
    let plain = secret_store::decrypt_with_passphrase(&encrypted, &salt, &nonce, wallet_passphrase)?;
    let phrase = String::from_utf8(plain)
        .map_err(|e| AppError::other(format!("backup mnemonic utf8: {e}")))?;
    Ok(Some(phrase))
}
