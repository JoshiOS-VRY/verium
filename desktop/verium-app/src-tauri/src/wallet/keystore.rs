//! Encrypted local HD wallet storage for light mode (mnemonic never sent to servers).

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::coin_profile::CoinId;
use crate::error::{AppError, AppResult};

const KEYSTORE_LABEL: &str = "light-wallet-keystore";

static SESSION_MNEMONICS: Lazy<Mutex<HashMap<String, Zeroizing<String>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightWalletRecord {
    pub coin: String,
    pub encrypted_mnemonic: String,
    pub salt: String,
    pub nonce: String,
    pub created_at: u64,
    pub next_receive_index: u32,
    #[serde(default)]
    pub label: String,
    /// Watch-only script pubkeys for balance/history while the wallet is locked.
    #[serde(default)]
    pub cached_script_hexes: Vec<String>,
    /// True after Electrum gap scan has persisted the full address range.
    #[serde(default)]
    pub addresses_scan_complete: bool,
    /// Scripts known to hold funds — balance queries use this instead of the full precache list.
    #[serde(default)]
    pub funded_script_hexes: Vec<String>,
    /// Resume cursor for chunked initial Electrum indexing (precache bootstrap).
    #[serde(default)]
    pub index_precache_offset: u32,
    /// Resume cursor for external-chain gap scan.
    #[serde(default)]
    pub index_gap_external: u32,
    /// True when external gap scan has hit the gap limit (distinct from cursor 0).
    #[serde(default)]
    pub index_gap_external_done: bool,
    /// Resume cursor for internal-chain gap scan.
    #[serde(default)]
    pub index_gap_internal: u32,
    /// True when internal gap scan has hit the gap limit.
    #[serde(default)]
    pub index_gap_internal_done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LightKeystore {
    pub wallets: HashMap<String, LightWalletRecord>,
    #[serde(default)]
    pub unlocked_until_by_coin: HashMap<String, u64>,
}

fn keystore_path() -> std::path::PathBuf {
    crate::config::app_config_base().join("light-keystore.json")
}

fn encrypted_keystore_path() -> std::path::PathBuf {
    crate::config::app_config_base()
        .join("secure")
        .join(format!("{KEYSTORE_LABEL}.enc"))
}

fn encrypted_keystore_backup_path() -> std::path::PathBuf {
    crate::config::app_config_base()
        .join("secure")
        .join(format!("{KEYSTORE_LABEL}.enc.bak"))
}

/// Windows Credential Manager resets can orphan encrypted blobs; restore the last backup.
fn try_restore_encrypted_keystore_backup() {
    let enc = encrypted_keystore_path();
    if enc.exists() {
        return;
    }
    let bak = encrypted_keystore_backup_path();
    if !bak.exists() {
        return;
    }
    if let Some(parent) = enc.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match std::fs::copy(&bak, &enc) {
        Ok(_) => tracing::info!("restored light-wallet-keystore from encrypted backup"),
        Err(e) => tracing::warn!("could not restore light-wallet-keystore backup: {e}"),
    }
}

fn load_keystore_from_plaintext(path: &std::path::Path) -> AppResult<Option<LightKeystore>> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(path)
        .map_err(|e| AppError::other(format!("could not read light keystore: {e}")))?;
    if raw.trim().is_empty() {
        return Ok(None);
    }
    serde_json::from_str(&raw)
        .map(Some)
        .map_err(|e| AppError::other(format!("light keystore JSON is invalid: {e}")))
}

pub fn load_keystore() -> AppResult<LightKeystore> {
    let path = keystore_path();
    try_restore_encrypted_keystore_backup();
    let encrypted = crate::secret_store::load_json(
        KEYSTORE_LABEL,
        &path,
        LightKeystore::default(),
    )?;
    if !encrypted.wallets.is_empty() {
        return Ok(encrypted);
    }
    if let Some(legacy) = load_keystore_from_plaintext(&path)? {
        tracing::info!("migrating legacy plaintext light-keystore.json to encrypted store");
        save_keystore(&legacy)?;
        let _ = std::fs::remove_file(&path);
        return Ok(legacy);
    }
    Ok(encrypted)
}

pub fn save_keystore(store: &LightKeystore) -> AppResult<()> {
    if let Some(parent) = encrypted_keystore_path().parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    crate::secret_store::save_json(KEYSTORE_LABEL, store)?;
    let path = keystore_path();
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
    verify_keystore_persisted(store)?;
    Ok(())
}

fn verify_keystore_persisted(expected: &LightKeystore) -> AppResult<()> {
    if expected.wallets.is_empty() {
        return Ok(());
    }
    let loaded = crate::secret_store::load_json_strict::<LightKeystore>(KEYSTORE_LABEL)?;
    for coin in expected.wallets.keys() {
        if !loaded.wallets.contains_key(coin) {
            return Err(AppError::other(format!(
                "light wallet file did not persist for {coin} — check disk space and Windows Credential Manager (service: com.vericonomy.wallet.desktop), then retry"
            )));
        }
    }
    Ok(())
}

pub fn wallet_exists(coin: CoinId) -> AppResult<bool> {
    let store = load_keystore()?;
    Ok(store.wallets.contains_key(coin.as_str()))
}

pub fn cached_script_hexes(coin: CoinId) -> AppResult<Vec<String>> {
    let store = load_keystore()?;
    Ok(store
        .wallets
        .get(coin.as_str())
        .map(|r| r.cached_script_hexes.clone())
        .unwrap_or_default())
}

pub fn clear_cached_script_hexes(coin: CoinId) -> AppResult<()> {
    let mut store = load_keystore()?;
    let Some(record) = store.wallets.get_mut(coin.as_str()) else {
        return Ok(());
    };
    if record.cached_script_hexes.is_empty() && record.addresses_scan_complete {
        return Ok(());
    }
    record.cached_script_hexes.clear();
    record.funded_script_hexes.clear();
    record.addresses_scan_complete = false;
    record.index_precache_offset = 0;
    record.index_gap_external = 0;
    record.index_gap_external_done = false;
    record.index_gap_internal = 0;
    record.index_gap_internal_done = false;
    save_keystore(&store)
}

pub fn funded_script_hexes(coin: CoinId) -> AppResult<Vec<String>> {
    let store = load_keystore()?;
    Ok(store
        .wallets
        .get(coin.as_str())
        .map(|r| r.funded_script_hexes.clone())
        .unwrap_or_default())
}

pub fn set_funded_script_hexes(coin: CoinId, scripts: &[String]) -> AppResult<()> {
    let mut store = load_keystore()?;
    let record = store
        .wallets
        .get_mut(coin.as_str())
        .ok_or_else(|| AppError::other("light wallet not found"))?;
    record.funded_script_hexes = scripts.to_vec();
    save_keystore(&store)
}

pub fn set_cached_script_hexes(coin: CoinId, scripts: &[String]) -> AppResult<()> {
    set_cached_script_hexes_with_scan_flag(coin, scripts, false)
}

pub fn set_cached_script_hexes_after_scan(coin: CoinId, scripts: &[String]) -> AppResult<()> {
    set_cached_script_hexes_with_scan_flag(coin, scripts, true)
}

fn set_cached_script_hexes_with_scan_flag(
    coin: CoinId,
    scripts: &[String],
    scan_complete: bool,
) -> AppResult<()> {
    if scripts.is_empty() {
        return Ok(());
    }
    let mut store = load_keystore()?;
    let record = store
        .wallets
        .get_mut(coin.as_str())
        .ok_or_else(|| AppError::other("light wallet not found"))?;
    record.cached_script_hexes = scripts.to_vec();
    if scan_complete {
        record.addresses_scan_complete = true;
    }
    save_keystore(&store)
}

#[derive(Debug, Clone, Copy, Default)]
pub struct IndexingProgress {
    pub precache_offset: u32,
    pub gap_external: u32,
    pub gap_external_done: bool,
    pub gap_internal: u32,
    pub gap_internal_done: bool,
}

pub fn indexing_progress(coin: CoinId) -> AppResult<IndexingProgress> {
    let store = load_keystore()?;
    Ok(store
        .wallets
        .get(coin.as_str())
        .map(|r| {
            let mut progress = IndexingProgress {
                precache_offset: r.index_precache_offset,
                gap_external: r.index_gap_external,
                gap_external_done: r.index_gap_external_done,
                gap_internal: r.index_gap_internal,
                gap_internal_done: r.index_gap_internal_done,
            };
            // Wallets that started internal scan before gap_external_done existed:
            // external had finished (old code reset cursor to 0).
            if !progress.gap_external_done && progress.gap_internal > 0 {
                progress.gap_external_done = true;
            }
            progress
        })
        .unwrap_or_default())
}

pub fn set_indexing_progress(coin: CoinId, progress: IndexingProgress) -> AppResult<()> {
    let mut store = load_keystore()?;
    let record = store
        .wallets
        .get_mut(coin.as_str())
        .ok_or_else(|| AppError::other("light wallet not found"))?;
    record.index_precache_offset = progress.precache_offset;
    record.index_gap_external = progress.gap_external;
    record.index_gap_external_done = progress.gap_external_done;
    record.index_gap_internal = progress.gap_internal;
    record.index_gap_internal_done = progress.gap_internal_done;
    save_keystore(&store)
}

pub fn reset_indexing_progress(coin: CoinId) -> AppResult<()> {
    set_indexing_progress(
        coin,
        IndexingProgress {
            precache_offset: 0,
            gap_external: 0,
            gap_external_done: false,
            gap_internal: 0,
            gap_internal_done: false,
        },
    )
}

pub fn mark_address_scan_incomplete(coin: CoinId) -> AppResult<()> {
    let mut store = load_keystore()?;
    let Some(record) = store.wallets.get_mut(coin.as_str()) else {
        return Ok(());
    };
    record.addresses_scan_complete = false;
    record.index_precache_offset = 0;
    record.index_gap_external = 0;
    record.index_gap_external_done = false;
    record.index_gap_internal = 0;
    record.index_gap_internal_done = false;
    save_keystore(&store)
}

pub fn mark_address_scan_complete(coin: CoinId) -> AppResult<()> {
    let mut store = load_keystore()?;
    let Some(record) = store.wallets.get_mut(coin.as_str()) else {
        return Ok(());
    };
    record.addresses_scan_complete = true;
    record.index_precache_offset = 0;
    record.index_gap_external = 0;
    record.index_gap_external_done = true;
    record.index_gap_internal = 0;
    record.index_gap_internal_done = true;
    save_keystore(&store)
}

pub fn needs_full_address_scan(coin: CoinId) -> AppResult<bool> {
    let store = load_keystore()?;
    let Some(record) = store.wallets.get(coin.as_str()) else {
        return Ok(false);
    };
    Ok(!record.addresses_scan_complete)
}

fn seal_wallet_record(
    coin: CoinId,
    seed_secret: &str,
    passphrase: &str,
    label: Option<&str>,
    preserve_created_at: Option<u64>,
) -> AppResult<LightWalletRecord> {
    let (encrypted, salt, nonce) =
        crate::secret_store::encrypt_with_passphrase(seed_secret.as_bytes(), passphrase)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    Ok(LightWalletRecord {
        coin: coin.as_str().to_string(),
        encrypted_mnemonic: hex::encode(encrypted),
        salt: hex::encode(salt),
        nonce: hex::encode(nonce),
        created_at: preserve_created_at.unwrap_or(now),
        next_receive_index: 0,
        label: label.unwrap_or("").to_string(),
        cached_script_hexes: Vec::new(),
        addresses_scan_complete: false,
        funded_script_hexes: Vec::new(),
        index_precache_offset: 0,
        index_gap_external: 0,
        index_gap_external_done: false,
        index_gap_internal: 0,
        index_gap_internal_done: false,
    })
}

fn persist_unlock_in_store(
    store: &mut LightKeystore,
    coin: CoinId,
    seed_secret: &str,
    seconds: u32,
) -> AppResult<()> {
    let until = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        + seconds as u64;
    store
        .unlocked_until_by_coin
        .insert(coin.as_str().to_string(), until);
    if let Ok(mut session) = SESSION_MNEMONICS.lock() {
        session.insert(
            coin.as_str().to_string(),
            Zeroizing::new(seed_secret.to_string()),
        );
    }
    Ok(())
}

fn clear_unlock_session(coin: CoinId) {
    if let Ok(mut session) = SESSION_MNEMONICS.lock() {
        session.remove(coin.as_str());
    }
}

pub fn create_wallet(
    coin: CoinId,
    mnemonic: &str,
    passphrase: &str,
    label: Option<&str>,
) -> AppResult<()> {
    let mut store = load_keystore()?;
    if store.wallets.contains_key(coin.as_str()) {
        return Err(AppError::other(format!(
            "{} light wallet already exists",
            coin.display_name()
        )));
    }
    let record = seal_wallet_record(coin, mnemonic, passphrase, label, None)?;
    store.wallets.insert(coin.as_str().to_string(), record);
    persist_unlock_in_store(&mut store, coin, mnemonic, 24 * 60 * 60)?;
    save_keystore(&store)?;
    let _ = crate::wallet::hd::precache_light_wallet_scripts(coin, mnemonic);
    Ok(())
}

/// Replace (or create) a light wallet from a BIP39 phrase or xprv master key.
pub fn import_wallet(
    coin: CoinId,
    seed_secret: &str,
    passphrase: &str,
    label: Option<&str>,
) -> AppResult<()> {
    clear_unlock_session(coin);
    let mut store = load_keystore()?;
    store.unlocked_until_by_coin.remove(coin.as_str());
    let created_at = store
        .wallets
        .get(coin.as_str())
        .map(|r| r.created_at);
    let record = seal_wallet_record(coin, seed_secret, passphrase, label, created_at)?;
    store.wallets.insert(coin.as_str().to_string(), record);
    persist_unlock_in_store(&mut store, coin, seed_secret, 24 * 60 * 60)?;
    save_keystore(&store)?;
    let _ = crate::wallet::cache::clear_coin_cache(coin);
    let _ = crate::wallet::hd::precache_light_wallet_scripts(coin, seed_secret);
    Ok(())
}

pub fn unlock_wallet(coin: CoinId, passphrase: &str, seconds: u32) -> AppResult<()> {
    let mut store = load_keystore()?;
    let record = store
        .wallets
        .get(coin.as_str())
        .ok_or_else(|| AppError::other("light wallet not found"))?;
    let phrase = decrypt_mnemonic(record, passphrase)?;
    persist_unlock_in_store(&mut store, coin, &phrase, seconds)?;
    save_keystore(&store)?;
    Ok(())
}

pub fn lock_wallet(coin: CoinId) -> AppResult<()> {
    clear_unlock_session(coin);
    let mut store = load_keystore()?;
    if store.unlocked_until_by_coin.remove(coin.as_str()).is_some() {
        save_keystore(&store)?;
    }
    Ok(())
}

/// True when the decrypted seed is in memory and can be used to sign.
pub fn signing_session_active(coin: CoinId) -> bool {
    SESSION_MNEMONICS
        .lock()
        .ok()
        .and_then(|session| session.get(coin.as_str()).map(|_| true))
        .unwrap_or(false)
}

pub fn is_unlocked(coin: CoinId) -> AppResult<bool> {
    let store = load_keystore()?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    Ok(store
        .unlocked_until_by_coin
        .get(coin.as_str())
        .map(|u| *u > now)
        .unwrap_or(false))
}

/// Decrypt the mnemonic from disk — never uses the in-memory session cache.
pub fn decrypt_mnemonic_for_send(coin: CoinId, passphrase: &str) -> AppResult<String> {
    if passphrase.trim().is_empty() {
        return Err(AppError::other(
            "Wallet passphrase is required to send.",
        ));
    }
    let store = load_keystore()?;
    let record = store
        .wallets
        .get(coin.as_str())
        .ok_or_else(|| AppError::other("light wallet not found"))?;
    decrypt_mnemonic(record, passphrase)
}

pub fn verify_passphrase(coin: CoinId, passphrase: &str) -> AppResult<()> {
    decrypt_mnemonic_for_send(coin, passphrase)?;
    Ok(())
}

pub fn unlocked_mnemonic(coin: CoinId, passphrase: &str) -> AppResult<String> {
    if let Ok(session) = SESSION_MNEMONICS.lock() {
        if let Some(phrase) = session.get(coin.as_str()) {
            return Ok(phrase.to_string());
        }
    }
    if passphrase.is_empty() {
        return Err(AppError::other(
            "Light wallet is locked — unlock it before sending (Transactions or Dashboard).",
        ));
    }
    let store = load_keystore()?;
    let record = store
        .wallets
        .get(coin.as_str())
        .ok_or_else(|| AppError::other("light wallet not found"))?;
    let phrase = decrypt_mnemonic(record, passphrase)?;
    if is_unlocked(coin)? {
        if let Ok(mut session) = SESSION_MNEMONICS.lock() {
            session.insert(
                coin.as_str().to_string(),
                Zeroizing::new(phrase.clone()),
            );
        }
    }
    Ok(phrase)
}

fn decrypt_mnemonic(record: &LightWalletRecord, passphrase: &str) -> AppResult<String> {
    let encrypted = hex::decode(record.encrypted_mnemonic.trim())
        .map_err(|e| AppError::other(format!("keystore corrupt: {e}")))?;
    let salt = hex::decode(record.salt.trim())
        .map_err(|e| AppError::other(format!("keystore corrupt: {e}")))?;
    let nonce = hex::decode(record.nonce.trim())
        .map_err(|e| AppError::other(format!("keystore corrupt: {e}")))?;
    let plain = crate::secret_store::decrypt_with_passphrase(&encrypted, &salt, &nonce, passphrase)?;
    let mut phrase = String::from_utf8(plain)
        .map_err(|e| AppError::other(format!("mnemonic utf8: {e}")))?;
    Ok(phrase)
}

pub fn bump_receive_index(coin: CoinId) -> AppResult<u32> {
    let mut store = load_keystore()?;
    let record = store
        .wallets
        .get_mut(coin.as_str())
        .ok_or_else(|| AppError::other("light wallet not found"))?;
    let idx = record.next_receive_index;
    record.next_receive_index = idx.saturating_add(1);
    save_keystore(&store)?;
    Ok(idx)
}

pub fn peek_receive_index(coin: CoinId) -> AppResult<u32> {
    let store = load_keystore()?;
    store
        .wallets
        .get(coin.as_str())
        .map(|r| r.next_receive_index)
        .ok_or_else(|| AppError::other("light wallet not found"))
}
