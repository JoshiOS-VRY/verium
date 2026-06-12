//! Encrypted local HD wallet storage for light mode (mnemonic never sent to servers).

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::coin_profile::CoinId;
use crate::error::{AppError, AppResult};

const KEYSTORE_LABEL: &str = "light-wallet-keystore";

pub const LIGHT_WALLET_RECOVERY_REQUIRED_MSG: &str = "Light wallet metadata is on this device but wallet keys are missing from the local store. Import your recovery phrase from Setup to restore access.";

pub fn light_wallet_usable(coin: CoinId) -> bool {
    light_keystore_health(coin) == LightKeystoreHealth::Ok
}

/// Quarantine orphaned CM-only encrypted blobs so optional mirrors and prefs can recover.
pub fn prepare_keystore_for_write() -> AppResult<()> {
    if crate::secret_store::encrypted_data_orphaned() {
        let _ = crate::secret_store::quarantine_orphaned_encrypted_data()?;
        invalidate_keystore_cache();
    }
    Ok(())
}

pub fn json_keystore_on_disk() -> bool {
    keystore_path().exists()
}

static SESSION_MNEMONICS: Lazy<Mutex<HashMap<String, Zeroizing<String>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// In-memory cache of the decrypted-at-rest keystore *envelope*. The cached
/// struct only ever holds ciphertext (`encrypted_mnemonic`) + metadata — the
/// plaintext seed lives solely in `SESSION_MNEMONICS` — so this is safe to keep
/// in memory. It removes the per-read decrypt that hot paths (wallet-info polls,
/// sync, signing-session checks) otherwise pay. Invalidated on every save.
const KEYSTORE_CACHE_TTL: Duration = Duration::from_secs(3);

static KEYSTORE_CACHE: Lazy<Mutex<Option<(Instant, LightKeystore)>>> =
    Lazy::new(|| Mutex::new(None));

fn cached_keystore() -> Option<LightKeystore> {
    let guard = KEYSTORE_CACHE.lock().ok()?;
    let (at, store) = guard.as_ref()?;
    if at.elapsed() < KEYSTORE_CACHE_TTL {
        Some(store.clone())
    } else {
        None
    }
}

fn store_keystore_cache(store: &LightKeystore) {
    if let Ok(mut guard) = KEYSTORE_CACHE.lock() {
        *guard = Some((Instant::now(), store.clone()));
    }
}

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

fn manifest_path() -> std::path::PathBuf {
    crate::config::app_config_base().join("light-wallet-manifest.json")
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct LightKeystoreManifest {
    /// coin id → created_at (unix seconds)
    wallets: HashMap<String, u64>,
    updated_at: u64,
}

fn write_manifest(store: &LightKeystore) -> AppResult<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let manifest = LightKeystoreManifest {
        wallets: store
            .wallets
            .iter()
            .map(|(coin, record)| (coin.clone(), record.created_at))
            .collect(),
        updated_at: now,
    };
    let path = manifest_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let tmp = path.with_extension("json.new");
    std::fs::write(&tmp, serde_json::to_string_pretty(&manifest)?)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

fn manifest_lists_coin(coin: CoinId) -> bool {
    let path = manifest_path();
    if !path.exists() {
        return false;
    }
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return false;
    };
    serde_json::from_str::<LightKeystoreManifest>(&raw)
        .ok()
        .is_some_and(|m| m.wallets.contains_key(coin.as_str()))
}

fn encrypted_keystore_on_disk() -> bool {
    encrypted_keystore_path().exists()
}

pub fn invalidate_keystore_cache() {
    if let Ok(mut guard) = KEYSTORE_CACHE.lock() {
        *guard = None;
    }
}

/// If the primary encrypted blob is missing, restore from `.enc.bak`.
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

/// When the primary blob exists but cannot be decrypted, try the backup copy.
fn try_swap_keystore_from_backup_on_decrypt_failure() {
    let enc = encrypted_keystore_path();
    let bak = encrypted_keystore_backup_path();
    if !enc.exists() || !bak.exists() {
        return;
    }
    if crate::secret_store::blob_readable(KEYSTORE_LABEL) {
        return;
    }
    if !crate::secret_store::blob_backup_readable(KEYSTORE_LABEL) {
        return;
    }
    let corrupt = enc.with_extension("enc.corrupt");
    let _ = std::fs::rename(&enc, &corrupt);
    match std::fs::copy(&bak, &enc) {
        Ok(_) => {
            tracing::warn!(
                "light-wallet-keystore primary blob unreadable; restored from backup"
            );
            invalidate_keystore_cache();
        }
        Err(e) => {
            tracing::warn!("could not restore light-wallet-keystore from backup: {e}");
            let _ = std::fs::rename(&corrupt, &enc);
        }
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

fn try_load_keystore_from_secret_store() -> AppResult<Option<LightKeystore>> {
    if !encrypted_keystore_on_disk() {
        return Ok(None);
    }
    if !crate::secret_store::blob_readable(KEYSTORE_LABEL) {
        return Ok(None);
    }
    let store = crate::secret_store::load_json_strict::<LightKeystore>(KEYSTORE_LABEL)?;
    if store.wallets.is_empty() {
        Ok(None)
    } else {
        Ok(Some(store))
    }
}

fn save_keystore_to_json(store: &LightKeystore) -> AppResult<()> {
    let path = keystore_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.new");
    std::fs::write(&tmp, serde_json::to_string_pretty(store)?).map_err(|e| {
        AppError::other(format!("could not write light keystore: {e}"))
    })?;
    let read_back = std::fs::read_to_string(&tmp).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        AppError::other(format!("could not read back light keystore: {e}"))
    })?;
    let parsed: LightKeystore = serde_json::from_str(&read_back).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        AppError::other(format!("light keystore post-write JSON invalid: {e}"))
    })?;
    if parsed.wallets.len() != store.wallets.len() {
        let _ = std::fs::remove_file(&tmp);
        return Err(AppError::other(
            "light keystore post-write verification failed (wallet count mismatch)",
        ));
    }
    std::fs::rename(&tmp, &path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        AppError::other(format!("could not finalize light keystore: {e}"))
    })?;
    Ok(())
}

fn archive_legacy_encrypted_keystore() {
    let enc = encrypted_keystore_path();
    if !enc.exists() {
        return;
    }
    let legacy = enc.with_extension("enc.legacy");
    if legacy.exists() {
        let _ = std::fs::remove_file(&legacy);
    }
    match std::fs::rename(&enc, &legacy) {
        Ok(_) => tracing::info!(
            "archived CM-wrapped light-wallet-keystore to {}",
            legacy.display()
        ),
        Err(e) => tracing::warn!("could not archive legacy encrypted keystore: {e}"),
    }
    let bak = encrypted_keystore_backup_path();
    if bak.exists() {
        let _ = std::fs::remove_file(bak);
    }
}

pub fn keystore_data_readable() -> bool {
    if let Ok(Some(store)) = load_keystore_from_plaintext(&keystore_path()) {
        if !store.wallets.is_empty() {
            return true;
        }
    }
    crate::secret_store::blob_readable(KEYSTORE_LABEL)
}

pub fn load_keystore() -> AppResult<LightKeystore> {
    if let Some(store) = cached_keystore() {
        return Ok(store);
    }
    let store = load_keystore_uncached()?;
    store_keystore_cache(&store);
    Ok(store)
}

fn load_keystore_uncached() -> AppResult<LightKeystore> {
    try_restore_encrypted_keystore_backup();
    try_swap_keystore_from_backup_on_decrypt_failure();

    // Primary: light-keystore.json (mnemonics are passphrase-encrypted inside).
    if let Some(store) = load_keystore_from_plaintext(&keystore_path())? {
        if !store.wallets.is_empty() {
            return Ok(store);
        }
    }

    // One-time migration from legacy CM-wrapped blob.
    if let Some(store) = try_load_keystore_from_secret_store()? {
        tracing::info!("migrating light-wallet-keystore from CM blob to light-keystore.json");
        save_keystore_to_json(&store)?;
        archive_legacy_encrypted_keystore();
        return Ok(store);
    }

    if encrypted_keystore_on_disk() {
        tracing::warn!(
            "legacy CM-wrapped light keystore exists but cannot be decrypted — \
             import recovery phrase to rebuild light-keystore.json"
        );
    }
    Ok(LightKeystore::default())
}

pub fn save_keystore(store: &LightKeystore) -> AppResult<()> {
    save_keystore_to_json(store)?;
    verify_keystore_persisted(store)?;
    write_manifest(store)?;
    // Optional CM mirror for upgrades from older builds; wallet does not depend on it.
    if !crate::secret_store::encrypted_data_orphaned() {
        if let Err(e) = crate::secret_store::save_json(KEYSTORE_LABEL, store) {
            tracing::debug!("optional CM keystore mirror skipped: {e}");
        }
    }
    store_keystore_cache(store);
    Ok(())
}

fn verify_keystore_persisted(expected: &LightKeystore) -> AppResult<()> {
    if expected.wallets.is_empty() {
        return Ok(());
    }
    let loaded = load_keystore_from_plaintext(&keystore_path())?
        .ok_or_else(|| AppError::other("light keystore did not persist to disk"))?;
    for coin in expected.wallets.keys() {
        if !loaded.wallets.contains_key(coin) {
            return Err(AppError::other(format!(
                "light wallet file did not persist for {coin} — check disk space, then retry"
            )));
        }
    }
    Ok(())
}

fn light_wallet_cache_exists(coin: CoinId) -> bool {
    crate::config::app_config_base()
        .join(format!("light-cache-{}.sqlite", coin.as_str()))
        .exists()
}

/// Whether a light wallet for this coin is recorded on disk (no CM decrypt).
fn keystore_backing_exists() -> bool {
    encrypted_keystore_on_disk() || keystore_path().exists()
}

/// Remove stale manifest / orphaned encrypted blobs left by a crashed or partial install.
#[cfg(mobile)]
pub fn cleanup_stale_mobile_wallet_artifacts() -> AppResult<()> {
    if !keystore_backing_exists() {
        let path = manifest_path();
        if path.exists() {
            let _ = std::fs::remove_file(&path);
            tracing::info!("mobile: removed stale light-wallet manifest without keystore");
        }
    }
    if crate::secret_store::encrypted_data_orphaned() && !keystore_backing_exists() {
        match crate::secret_store::quarantine_orphaned_encrypted_data() {
            Ok(msg) => tracing::info!("mobile: {msg}"),
            Err(e) => tracing::warn!("mobile: could not quarantine orphaned encrypted blobs: {e}"),
        }
        invalidate_keystore_cache();
    }
    Ok(())
}

#[cfg(not(mobile))]
pub fn cleanup_stale_mobile_wallet_artifacts() -> AppResult<()> {
    Ok(())
}

pub fn light_wallet_on_disk(coin: CoinId) -> bool {
    if !keystore_backing_exists() {
        return false;
    }
    manifest_lists_coin(coin) || light_wallet_cache_exists(coin)
}

pub fn manifest_lists_coin_for_diagnostics(coin: CoinId) -> bool {
    manifest_lists_coin(coin)
}

pub fn light_cache_exists_for_diagnostics(coin: CoinId) -> bool {
    light_wallet_cache_exists(coin)
}

pub fn encrypted_blob_exists_for_diagnostics() -> bool {
    encrypted_keystore_on_disk()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LightKeystoreHealth {
    Ok,
    Unreadable,
    Missing,
}

impl LightKeystoreHealth {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Unreadable => "unreadable",
            Self::Missing => "missing",
        }
    }
}

/// Decrypt health for a coin's light keystore (presence uses [`light_wallet_on_disk`]).
pub fn light_keystore_health(coin: CoinId) -> LightKeystoreHealth {
    if !keystore_backing_exists() {
        return LightKeystoreHealth::Missing;
    }
    if !manifest_lists_coin(coin) && !light_wallet_cache_exists(coin) {
        return LightKeystoreHealth::Missing;
    }
    match load_keystore() {
        Ok(store) if store.wallets.contains_key(coin.as_str()) => LightKeystoreHealth::Ok,
        Ok(_) | Err(_) => LightKeystoreHealth::Unreadable,
    }
}

pub fn wallet_exists(coin: CoinId) -> AppResult<bool> {
    let store = load_keystore()?;
    Ok(store.wallets.contains_key(coin.as_str()))
}

/// Refresh the plaintext manifest from a readable encrypted keystore (startup / migration).
pub fn sync_manifest_from_keystore() {
    invalidate_keystore_cache();
    if let Ok(store) = load_keystore() {
        if !store.wallets.is_empty() {
            let _ = write_manifest(&store);
            return;
        }
    }
    if manifest_path().exists() {
        return;
    }
    let mut manifest = LightKeystoreManifest::default();
    for coin in CoinId::all() {
        if light_wallet_cache_exists(*coin) {
            manifest.wallets.insert(coin.as_str().to_string(), 0);
        }
    }
    if manifest.wallets.is_empty() {
        return;
    }
    manifest.updated_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if let Ok(json) = serde_json::to_string_pretty(&manifest) {
        let path = manifest_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let tmp = path.with_extension("json.new");
        if std::fs::write(&tmp, json).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
            tracing::info!(
                "backfilled light-wallet-manifest.json from existing light wallet caches"
            );
        }
    }
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
    // Skip the encrypt+persist if nothing actually changed — gap-scan slices can
    // re-report the same cursor, and each save_keystore is an encrypt + verify.
    if record.index_precache_offset == progress.precache_offset
        && record.index_gap_external == progress.gap_external
        && record.index_gap_external_done == progress.gap_external_done
        && record.index_gap_internal == progress.gap_internal
        && record.index_gap_internal_done == progress.gap_internal_done
    {
        return Ok(());
    }
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
    prepare_keystore_for_write()?;
    let mut store = load_keystore()?;
    if store.wallets.contains_key(coin.as_str()) || light_wallet_on_disk(coin) {
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
    prepare_keystore_for_write()?;
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
        .ok_or_else(|| {
            if light_wallet_on_disk(coin) {
                AppError::other(LIGHT_WALLET_RECOVERY_REQUIRED_MSG)
            } else {
                AppError::other("light wallet not found")
            }
        })?;
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
    let store = load_keystore()?;
    let record = store
        .wallets
        .get(coin.as_str())
        .ok_or_else(|| AppError::other("light wallet not found"))?;
    decrypt_mnemonic(record, passphrase)?;
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
    let phrase = String::from_utf8(plain)
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

#[cfg(test)]
mod presence_tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn write_manifest(coins: &[&str]) {
        let path = manifest_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut wallets = HashMap::new();
        for coin in coins {
            wallets.insert((*coin).to_string(), 1_u64);
        }
        let manifest = LightKeystoreManifest {
            wallets,
            updated_at: 1,
        };
        let tmp = path.with_extension("json.test");
        let mut f = fs::File::create(&tmp).unwrap();
        write!(f, "{}", serde_json::to_string(&manifest).unwrap()).unwrap();
        drop(f);
        let _ = fs::rename(&tmp, &path);
    }

    fn cleanup_manifest() {
        let _ = fs::remove_file(manifest_path());
    }

    #[test]
    fn manifest_without_keystore_backing_is_not_on_disk() {
        write_manifest(&["verium"]);
        assert!(manifest_lists_coin(CoinId::Verium));
        assert!(!light_wallet_on_disk(CoinId::Verium));
        assert_eq!(
            light_keystore_health(CoinId::Verium),
            LightKeystoreHealth::Missing
        );
        cleanup_manifest();
    }

    #[test]
    fn light_keystore_health_ok_from_json_without_cm() {
        write_manifest(&["verium"]);
        let store = LightKeystore {
            wallets: HashMap::from([(
                "verium".to_string(),
                LightWalletRecord {
                    coin: "verium".to_string(),
                    encrypted_mnemonic: "deadbeef".into(),
                    salt: "00".into(),
                    nonce: "00".into(),
                    created_at: 1,
                    next_receive_index: 0,
                    label: String::new(),
                    cached_script_hexes: vec![],
                    addresses_scan_complete: false,
                    funded_script_hexes: vec![],
                    index_precache_offset: 0,
                    index_gap_external: 0,
                    index_gap_external_done: false,
                    index_gap_internal: 0,
                    index_gap_internal_done: false,
                },
            )]),
            unlocked_until_by_coin: HashMap::new(),
        };
        save_keystore_to_json(&store).unwrap();
        invalidate_keystore_cache();
        assert_eq!(
            light_keystore_health(CoinId::Verium),
            LightKeystoreHealth::Ok
        );
        let _ = fs::remove_file(keystore_path());
        cleanup_manifest();
    }
}
