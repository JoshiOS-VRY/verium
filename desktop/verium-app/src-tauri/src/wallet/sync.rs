//! Light-wallet chain sync: gap scan → funded scripts → UTXO set → balance.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex as StdMutex;
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;
use tokio::sync::Mutex;

use crate::chain::electrum::indexing::{RPC_BUDGET_PER_SYNC, SCRIPTS_PER_BATCH};
use crate::chain::types::WalletBalance;
use crate::chain::ChainBackend;
use crate::coin_profile::CoinId;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use async_trait::async_trait;

use crate::wallet::cache::LightWalletCache;
use crate::wallet::gap_scan_hook::GapScanHook;
use crate::wallet::explorer_history::fetch_wallet_history_from_explorer;
use crate::wallet::hd::{
    discover_script_hexes, enrich_utxo_addresses, resolve_addresses_for_script_hexes,
    uses_core_hd_paths,
};
use crate::wallet::keystore;

struct GapScanPersist<'a> {
    coin: CoinId,
    phrase: &'a str,
    backend: &'a dyn ChainBackend,
}

#[async_trait]
impl GapScanHook for GapScanPersist<'_> {
    async fn on_funded_batch(&self, funded: &[String], utxo_refresh: bool) -> AppResult<()> {
        if utxo_refresh {
            persist_funded_scripts(self.coin, self.phrase, self.backend, funded).await
        } else {
            keystore::set_funded_script_hexes(self.coin, funded)?;
            Ok(())
        }
    }
}

const GAP_LIMIT: u32 = 20;
const SYNC_TIMEOUT: Duration = Duration::from_secs(300);
/// Balance-first foreground poll — UTXO refresh only (no history RPC).
const BALANCE_REFRESH_MIN_INTERVAL: Duration = Duration::from_secs(3);
/// Foreground activity polling (receive / history).
const PENDING_REFRESH_MIN_INTERVAL: Duration = Duration::from_secs(2);

static SYNC_IN_FLIGHT: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));
static LAST_BALANCE_REFRESH: Lazy<StdMutex<HashMap<CoinId, Instant>>> =
    Lazy::new(|| StdMutex::new(HashMap::new()));
static LAST_PENDING_REFRESH: Lazy<StdMutex<HashMap<CoinId, Instant>>> =
    Lazy::new(|| StdMutex::new(HashMap::new()));

fn balance_refresh_due(coin: CoinId) -> bool {
    let mut map = match LAST_BALANCE_REFRESH.lock() {
        Ok(m) => m,
        Err(_) => return true,
    };
    let now = Instant::now();
    match map.get(&coin) {
        Some(last) if now.duration_since(*last) < BALANCE_REFRESH_MIN_INTERVAL => false,
        _ => {
            map.insert(coin, now);
            true
        }
    }
}

pub fn reset_balance_refresh_throttle(coin: CoinId) {
    if let Ok(mut map) = LAST_BALANCE_REFRESH.lock() {
        map.remove(&coin);
    }
}

/// True when a foreground balance refresh ran recently (UTO cache likely fresh).
pub fn is_utxo_cache_recent(coin: CoinId) -> bool {
    let Ok(map) = LAST_BALANCE_REFRESH.lock() else {
        return false;
    };
    map.get(&coin)
        .map(|last| last.elapsed() < BALANCE_REFRESH_MIN_INTERVAL)
        .unwrap_or(false)
}

fn pending_refresh_due(coin: CoinId) -> bool {
    let mut map = match LAST_PENDING_REFRESH.lock() {
        Ok(m) => m,
        Err(_) => return true,
    };
    let now = Instant::now();
    match map.get(&coin) {
        Some(last) if now.duration_since(*last) < PENDING_REFRESH_MIN_INTERVAL => false,
        _ => {
            map.insert(coin, now);
            true
        }
    }
}

pub fn reset_pending_refresh_throttle(coin: CoinId) {
    if let Ok(mut map) = LAST_PENDING_REFRESH.lock() {
        map.remove(&coin);
    }
}

/// Update the local UTXO cache immediately after a successful broadcast so
/// `getwalletinfo` reflects the spend before the next Electrum poll.
pub fn apply_local_send_cache_update(
    coin: CoinId,
    spent: &[crate::chain::types::Utxo],
    broadcast_txid: &str,
    signed_hex: &str,
    change_sats: i64,
    change_address: &str,
) -> AppResult<()> {
    use std::collections::HashSet;

    use crate::wallet::hd::address_to_script_pubkey;
    use crate::wallet::listtransactions_rows::rows_from_decoded_tx;
    use crate::wallet::utxo_selector::DUST_CHANGE_SATS;
    use crate::wallet::vericonomy_tx::decode_verium_tx;

    let raw = hex::decode(signed_hex.trim())
        .map_err(|e| crate::error::AppError::other(format!("post-send tx hex: {e}")))?;
    let tx = decode_verium_tx(&raw)?;
    let output_count = tx.outputs.len();

    let cache = LightWalletCache::open(coin)?;
    for utxo in spent {
        cache.remove_utxo(&utxo.txid, utxo.vout)?;
    }
    if change_sats > DUST_CHANGE_SATS && output_count > 0 {
        let script = address_to_script_pubkey(coin, change_address)?;
        let change_vout = (output_count - 1) as u32;
        cache.upsert_utxo(&crate::chain::types::Utxo {
            txid: broadcast_txid.to_string(),
            vout: change_vout,
            value_sats: change_sats,
            script_hex: hex::encode(script),
            height: 0,
            address: change_address.to_string(),
            confirmations: 0,
        })?;
        register_local_optimistic_utxo(coin, broadcast_txid, change_vout)?;
    }

    let mut wallet_addresses: HashSet<String> = spent
        .iter()
        .map(|u| u.address.clone())
        .filter(|a| !a.is_empty())
        .collect();
    if !change_address.is_empty() {
        wallet_addresses.insert(change_address.to_string());
    }

    let mut wallet_scripts: HashSet<Vec<u8>> = HashSet::new();
    for utxo in spent {
        if let Ok(bytes) = hex::decode(utxo.script_hex.trim()) {
            wallet_scripts.insert(bytes);
        }
    }
    if change_sats > DUST_CHANGE_SATS {
        if let Ok(script) = address_to_script_pubkey(coin, change_address) {
            wallet_scripts.insert(script);
        }
    }

    let mut prev_output_is_ours = HashSet::new();
    for utxo in spent {
        prev_output_is_ours.insert((utxo.txid.clone(), utxo.vout));
    }

    let history_rows = rows_from_decoded_tx(
        coin,
        broadcast_txid,
        &tx,
        0,
        None,
        &wallet_scripts,
        &wallet_addresses,
        &prev_output_is_ours,
    );
    append_local_tx_history_rows(coin, &history_rows)?;

    Ok(())
}

/// Merge freshly broadcast rows into cached history so sends appear immediately.
pub fn append_local_tx_history_rows(coin: CoinId, rows: &[crate::chain::types::WalletTx]) -> AppResult<()> {
    if rows.is_empty() {
        return Ok(());
    }
    let cache = LightWalletCache::open(coin)?;
    let existing = cache
        .list_tx_history(TX_HISTORY_CACHE_LIMIT)
        .unwrap_or_default();
    let merged = merge_tx_history(existing, rows);
    cache.replace_tx_history(&merged)?;
    Ok(())
}

pub fn balance_from_utxo_cache(coin: CoinId) -> WalletBalance {
    let mut confirmed = 0i64;
    let mut unconfirmed = 0i64;
    let Ok(cache) = LightWalletCache::open(coin) else {
        return WalletBalance {
            confirmed_sats: 0,
            unconfirmed_sats: 0,
            immature_sats: 0,
        };
    };
    let Ok(utxos) = cache.list_utxos() else {
        return WalletBalance {
            confirmed_sats: 0,
            unconfirmed_sats: 0,
            immature_sats: 0,
        };
    };

    let optimistic = local_optimistic_utxo_keys(coin);
    let electrum_keys = last_electrum_utxo_keys(coin);
    let electrum_snapshot_ready = has_last_electrum_utxo_snapshot(coin);
    let mut stale_unconfirmed: Vec<(String, u32)> = Vec::new();

    for utxo in utxos {
        if utxo.height > 0 {
            confirmed += utxo.value_sats;
            continue;
        }
        let key = (utxo.txid.clone(), utxo.vout);
        let trusted = optimistic.contains(&key)
            || (electrum_snapshot_ready && electrum_keys.contains(&key));
        if trusted {
            unconfirmed += utxo.value_sats;
        } else if electrum_snapshot_ready {
            stale_unconfirmed.push(key);
        } else {
            // Before the first Electrum UTXO snapshot, keep legacy behaviour.
            unconfirmed += utxo.value_sats;
        }
    }

    for (txid, vout) in stale_unconfirmed {
        if let Err(e) = cache.remove_utxo(&txid, vout) {
            tracing::debug!(
                "pruned stale unconfirmed utxo {}:{} for {}: {e}",
                txid,
                vout,
                coin.as_str()
            );
        } else {
            tracing::info!(
                "pruned stale unconfirmed utxo {}:{} for {}",
                txid,
                vout,
                coin.as_str()
            );
        }
    }

    WalletBalance {
        confirmed_sats: confirmed,
        unconfirmed_sats: unconfirmed,
        immature_sats: 0,
    }
}

/// One entry point for Electrum sync. Concurrent calls for the same coin are coalesced.
pub async fn sync_light_wallet(state: &AppState, coin: CoinId) -> AppResult<()> {
    let key = coin.as_str().to_string();
    {
        let mut in_flight = SYNC_IN_FLIGHT.lock().await;
        if !in_flight.insert(key.clone()) {
            return Ok(());
        }
    }
    let result = match tokio::time::timeout(SYNC_TIMEOUT, sync_light_wallet_inner(state, coin)).await
    {
        Ok(inner) => inner,
        Err(_) => {
            tracing::error!(
                "light wallet sync timed out after {}s for {}",
                SYNC_TIMEOUT.as_secs(),
                coin.as_str()
            );
            Err(crate::error::AppError::other(format!(
                "light wallet sync timed out after {}s",
                SYNC_TIMEOUT.as_secs()
            )))
        }
    };
    SYNC_IN_FLIGHT.lock().await.remove(&key);
    result
}

async fn sync_light_wallet_inner(state: &AppState, coin: CoinId) -> AppResult<()> {
    if !keystore::wallet_exists(coin)? {
        return Ok(());
    }

    let backend = crate::wallet::backend::resolve_backend(state, coin).await?;
    let needs_scan = keystore::needs_full_address_scan(coin)?;

    if needs_scan {
        if !keystore::is_unlocked(coin)? || !keystore::signing_session_active(coin) {
            return Ok(());
        }
        let phrase = keystore::unlocked_mnemonic(coin, "")?;
        run_gap_scan_and_persist(coin, &phrase, backend.as_ref()).await?;
        return Ok(());
    }

    let phrase = if keystore::is_unlocked(coin)? && keystore::signing_session_active(coin) {
        Some(keystore::unlocked_mnemonic(coin, "")?)
    } else {
        None
    };
    refresh_light_wallet_utxos_from_network(coin, phrase.as_deref(), backend.as_ref()).await?;
    let funded = keystore::funded_script_hexes(coin)?;
    // Explorer for confirmed history; Electrum merge keeps 0-conf rows fresh.
    refresh_tx_history_cache(coin, &funded, backend.as_ref(), true).await;
    Ok(())
}

/// Whether a `sync_light_wallet` call is in progress for this coin.
pub async fn is_light_wallet_sync_in_flight(coin: CoinId) -> bool {
    SYNC_IN_FLIGHT.lock().await.contains(coin.as_str())
}

/// Minimum wait between manual address rescans from Settings.
pub const MANUAL_RESCAN_COOLDOWN_SECS: u64 = 3600;
const LAST_MANUAL_RESCAN_META: &str = "last_manual_rescan_at";

fn unix_now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn cooldown_remaining_secs(last_at: u64, now: u64, cooldown_secs: u64) -> u64 {
    if now <= last_at {
        return cooldown_secs;
    }
    let elapsed = now - last_at;
    if elapsed >= cooldown_secs {
        0
    } else {
        cooldown_secs - elapsed
    }
}

pub fn manual_rescan_cooldown_remaining_secs(coin: CoinId) -> u64 {
    let Ok(cache) = LightWalletCache::open(coin) else {
        return 0;
    };
    let Ok(Some(ts)) = cache.get_meta(LAST_MANUAL_RESCAN_META) else {
        return 0;
    };
    let Ok(last_at) = ts.parse::<u64>() else {
        return 0;
    };
    cooldown_remaining_secs(last_at, unix_now_secs(), MANUAL_RESCAN_COOLDOWN_SECS)
}

fn had_manual_rescan(coin: CoinId) -> bool {
    LightWalletCache::open(coin)
        .ok()
        .and_then(|c| c.get_meta(LAST_MANUAL_RESCAN_META).ok().flatten())
        .is_some()
}

/// Guards manual rescan from Settings (cooldown, in-flight sync, initial scan).
pub async fn assert_manual_rescan_allowed(coin: CoinId) -> AppResult<()> {
    if !keystore::is_unlocked(coin)? {
        return Err(AppError::other(
            "unlock your light wallet before rescanning addresses",
        ));
    }
    if is_light_wallet_sync_in_flight(coin).await {
        return Err(AppError::other(
            "address rescan is already running — wait for it to finish",
        ));
    }
    let scan_incomplete = keystore::needs_full_address_scan(coin)?;
    if scan_incomplete && !had_manual_rescan(coin) {
        return Err(AppError::other(
            "initial address scan is still in progress — wait for it to finish",
        ));
    }
    let remaining = manual_rescan_cooldown_remaining_secs(coin);
    if remaining > 0 {
        let minutes = (remaining + 59) / 60;
        return Err(AppError::other(format!(
            "rescan was used recently — try again in about {minutes} minute{}",
            if minutes == 1 { "" } else { "s" }
        )));
    }
    Ok(())
}

/// Record cooldown and reset scan state before starting a manual rescan.
pub fn begin_manual_rescan(coin: CoinId) -> AppResult<()> {
    let cache = LightWalletCache::open(coin)?;
    cache.set_meta(LAST_MANUAL_RESCAN_META, &unix_now_secs().to_string())?;
    keystore::mark_address_scan_incomplete(coin)?;
    reset_balance_probe_state(coin)?;
    Ok(())
}

const PRECACHE_PROBE_COMPLETE_META: &str = "precache_probe_complete";

pub fn is_precache_probe_complete(coin: CoinId) -> bool {
    LightWalletCache::open(coin)
        .ok()
        .and_then(|c| c.get_meta(PRECACHE_PROBE_COMPLETE_META).ok().flatten())
        .map(|v| v == "1")
        .unwrap_or(false)
}

pub fn reset_balance_probe_state(coin: CoinId) -> AppResult<()> {
    if let Ok(cache) = LightWalletCache::open(coin) {
        let _ = cache.set_meta(PRECACHE_PROBE_COMPLETE_META, "0");
        let _ = cache.set_meta("precache_probe_offset", "0");
    }
    Ok(())
}

/// Brand-new BIP39 light wallets have no on-chain history — skip the multi-minute gap scan.
pub fn mark_new_wallet_setup_ready(coin: CoinId) -> AppResult<()> {
    keystore::mark_address_scan_complete(coin)?;
    mark_precache_probe_complete(coin);
    if let Ok(cache) = LightWalletCache::open(coin) {
        let _ = cache.replace_utxos(&[]);
        let _ = cache.set_meta("utxo_funded_count", "0");
    }
    Ok(())
}

fn mark_precache_probe_complete(coin: CoinId) {
    if let Ok(cache) = LightWalletCache::open(coin) {
        let _ = cache.set_meta(PRECACHE_PROBE_COMPLETE_META, "1");
    }
}

/// Refresh UTXOs for send using scripts already in the funded set / local cache.
/// Skips precache probing so send does not fan out 160+ balance RPCs before signing.
pub(crate) async fn refresh_send_utxos_from_network(
    coin: CoinId,
    phrase: Option<&str>,
    backend: &dyn ChainBackend,
) -> AppResult<()> {
    let mut scripts = keystore::funded_script_hexes(coin)?;
    if let Ok(cache) = LightWalletCache::open(coin) {
        if let Ok(utxos) = cache.list_utxos() {
            for utxo in utxos {
                if utxo.script_hex.is_empty() {
                    continue;
                }
                if scripts
                    .iter()
                    .any(|s| s.eq_ignore_ascii_case(&utxo.script_hex))
                {
                    continue;
                }
                scripts.push(utxo.script_hex.clone());
            }
        }
    }
    if scripts.is_empty() {
        return Ok(());
    }
    refresh_utxos(coin, &scripts, backend, phrase).await?;
    Ok(())
}

/// Discover funded scripts + refresh UTXO cache (no transaction history RPC).
pub(crate) async fn refresh_light_wallet_utxos_from_network(
    coin: CoinId,
    phrase: Option<&str>,
    backend: &dyn ChainBackend,
) -> AppResult<()> {
    let mut funded = keystore::funded_script_hexes(coin)?;
    extend_funded_from_precached_balances(coin, phrase, backend, &mut funded).await?;
    if funded.is_empty() {
        return Ok(());
    }
    refresh_utxos(coin, &funded, backend, phrase).await?;
    Ok(())
}

/// Foreground balance sync: probe precached addresses, refresh UTXOs, update cache.
pub async fn refresh_light_wallet_balance(state: &AppState, coin: CoinId) -> AppResult<()> {
    if !keystore::wallet_exists(coin)? || !keystore::is_unlocked(coin)? {
        return Ok(());
    }
    if !balance_refresh_due(coin) {
        return Ok(());
    }
    if SYNC_IN_FLIGHT.lock().await.contains(coin.as_str()) {
        return Ok(());
    }
    let phrase = if keystore::signing_session_active(coin) {
        Some(keystore::unlocked_mnemonic(coin, "")?)
    } else {
        None
    };
    let backend = crate::wallet::backend::resolve_backend(state, coin).await?;
    if let Err(e) =
        refresh_light_wallet_utxos_from_network(coin, phrase.as_deref(), backend.as_ref()).await
    {
        tracing::debug!("balance utxo refresh failed for {}: {e}", coin.as_str());
    } else if !keystore::needs_full_address_scan(coin)? && !is_precache_probe_complete(coin) {
        // Zero-balance wallets: gap scan already finished; no need to keep probing forever.
        mark_precache_probe_complete(coin);
    }
    Ok(())
}

/// Foreground refresh: UTXOs plus Electrum pending history (history / activity screens).
pub async fn refresh_light_wallet_pending(state: &AppState, coin: CoinId) -> AppResult<()> {
    if !keystore::wallet_exists(coin)? || !keystore::is_unlocked(coin)? {
        return Ok(());
    }
    if !pending_refresh_due(coin) {
        return Ok(());
    }
    let funded = keystore::funded_script_hexes(coin)?;
    if funded.is_empty() {
        return Ok(());
    }
    let backend = crate::wallet::backend::resolve_backend(state, coin).await?;
    if let Err(e) = refresh_utxos(coin, &funded, backend.as_ref(), None).await {
        tracing::debug!("pending utxo refresh failed for {}: {e}", coin.as_str());
    }
    refresh_pending_tx_history_only(coin, &funded, backend.as_ref()).await;
    Ok(())
}

/// Number of history rows kept in the local cache (matches UI list cap).
pub const TX_HISTORY_CACHE_LIMIT: usize = 500;

const PENDING_HISTORY_ENRICH_LIMIT: usize = 64;
const PENDING_HISTORY_FETCH_LIMIT: usize = 128;

struct ElectrumHistoryFetcher<'a>(&'a dyn ChainBackend);

#[async_trait]
impl crate::chain::electrum::history::HistoryTxFetcher for ElectrumHistoryFetcher<'_> {
    async fn fetch_raw_tx_hex(&self, txid: &str) -> AppResult<String> {
        self.0.get_raw_tx_hex(txid).await
    }
}

fn tx_history_confirmed(tx: &crate::chain::types::WalletTx) -> bool {
    tx.height > 0 || tx.blockheight.is_some()
}

/// Prefer confirmed rows over stale optimistic 0-conf copies for the same key.
fn merge_tx_history(base: Vec<crate::chain::types::WalletTx>, pending: &[crate::chain::types::WalletTx]) -> Vec<crate::chain::types::WalletTx> {
    use std::collections::HashMap;

    let mut map: HashMap<String, crate::chain::types::WalletTx> = base
        .into_iter()
        .map(|tx| (crate::wallet::listtransactions_rows::wallet_tx_row_key(&tx), tx))
        .collect();
    for tx in pending {
        let key = crate::wallet::listtransactions_rows::wallet_tx_row_key(tx);
        if let Some(existing) = map.get(&key) {
            if tx_history_confirmed(existing) && !tx_history_confirmed(tx) {
                continue;
            }
        }
        map.insert(key, tx.clone());
    }
    let mut rows: Vec<_> = map.into_values().collect();
    rows.sort_by(|a, b| {
        let ta = a.time.unwrap_or(0);
        let tb = b.time.unwrap_or(0);
        tb.cmp(&ta).then(b.height.cmp(&a.height))
    });
    rows.truncate(TX_HISTORY_CACHE_LIMIT);
    rows
}

async fn merge_pending_history_from_electrum(
    coin: CoinId,
    funded: &[String],
    backend: &dyn ChainBackend,
    base: Vec<crate::chain::types::WalletTx>,
) -> Option<Vec<crate::chain::types::WalletTx>> {
    use std::collections::HashSet;

    let mut history = backend
        .get_history_for_scripts(funded, PENDING_HISTORY_FETCH_LIMIT)
        .await
        .ok()?;
    let stale_txids: HashSet<String> = base
        .iter()
        .filter(|tx| tx.height <= 0 && !tx.txid.is_empty())
        .map(|tx| tx.txid.clone())
        .collect();
    history.retain(|tx| tx.height <= 0 || stale_txids.contains(&tx.txid));
    if history.is_empty() {
        return Some(base);
    }
    let enrich_limit = history.len().min(PENDING_HISTORY_ENRICH_LIMIT);
    let tip = backend.get_tip().await.ok().map(|t| t.height);
    let fetcher = ElectrumHistoryFetcher(backend);
    let expanded = crate::chain::electrum::history::expand_wallet_history_rows(
        coin,
        funded,
        &history,
        &fetcher,
        Some(enrich_limit),
        tip,
    )
    .await
    .ok()?;
    Some(merge_tx_history(base, &expanded))
}

async fn write_tx_history_cache(coin: CoinId, rows: &[crate::chain::types::WalletTx]) {
    if rows.is_empty() {
        return;
    }
    if let Ok(cache) = LightWalletCache::open(coin) {
        let pending_local: Vec<_> = cache
            .list_tx_history(TX_HISTORY_CACHE_LIMIT)
            .unwrap_or_default()
            .into_iter()
            .filter(|t| t.height <= 0)
            .collect();
        let merged = if pending_local.is_empty() {
            rows.to_vec()
        } else {
            merge_tx_history(rows.to_vec(), &pending_local)
        };
        if let Err(e) = cache.replace_tx_history(&merged) {
            tracing::debug!("history cache write failed for {}: {e}", coin.as_str());
        }
    }
}

async fn refresh_pending_tx_history_only(coin: CoinId, funded: &[String], backend: &dyn ChainBackend) {
    let tip = backend.get_tip().await.ok().map(|t| t.height);
    if let Ok(cache) = LightWalletCache::open(coin) {
        if let Some(height) = tip {
            let _ = cache.set_meta("tip_height", &height.to_string());
        }
    }
    let base = LightWalletCache::open(coin)
        .ok()
        .and_then(|cache| cache.list_tx_history(TX_HISTORY_CACHE_LIMIT).ok())
        .unwrap_or_default();
    if let Some(merged) = merge_pending_history_from_electrum(coin, funded, backend, base).await {
        write_tx_history_cache(coin, &merged).await;
    }
}

async fn refresh_tx_history_cache(
    coin: CoinId,
    funded: &[String],
    backend: &dyn ChainBackend,
    merge_pending_electrum: bool,
) {
    if funded.is_empty() {
        return;
    }

    let tip = backend.get_tip().await.ok().map(|t| t.height);
    if let Ok(cache) = LightWalletCache::open(coin) {
        if let Some(height) = tip {
            let _ = cache.set_meta("tip_height", &height.to_string());
        }
    }

    if let Ok(addresses) = addresses_for_history(coin) {
        if let Ok(history) =
            fetch_wallet_history_from_explorer(coin, &addresses, TX_HISTORY_CACHE_LIMIT, tip).await
        {
            let rows = if merge_pending_electrum {
                match merge_pending_history_from_electrum(coin, funded, backend, history.clone()).await
                {
                    Some(merged) => merged,
                    None => history,
                }
            } else {
                history
            };
            if !rows.is_empty() {
                write_tx_history_cache(coin, &rows).await;
            }
            return;
        }
    }

    // Explorer unavailable — merge Electrum pending only when UTXOs changed.
    if merge_pending_electrum {
        refresh_pending_tx_history_only(coin, funded, backend).await;
    }
}

async fn persist_funded_scripts(
    coin: CoinId,
    phrase: &str,
    backend: &dyn ChainBackend,
    funded: &[String],
) -> AppResult<()> {
    if funded.is_empty() {
        return Ok(());
    }
    keystore::set_funded_script_hexes(coin, funded)?;
    refresh_utxos(coin, funded, backend, Some(phrase)).await?;
    Ok(())
}

async fn bootstrap_precache_slice(
    coin: CoinId,
    phrase: &str,
    backend: &dyn ChainBackend,
    progress: &mut keystore::IndexingProgress,
    funded_so_far: &mut Vec<String>,
) -> AppResult<bool> {
    // BIP44 mnemonics: gap scan covers receive addresses; precache Electrum probe is redundant.
    if !uses_core_hd_paths(coin, phrase) {
        let precached = keystore::cached_script_hexes(coin)?;
        let end = precached.len() as u32;
        if progress.precache_offset < end {
            progress.precache_offset = end;
            keystore::set_indexing_progress(coin, *progress)?;
        }
        return Ok(true);
    }

    let precached = keystore::cached_script_hexes(coin)?;
    if precached.is_empty() {
        return Ok(true);
    }
    if progress.precache_offset as usize >= precached.len() {
        return Ok(true);
    }

    let start = progress.precache_offset as usize;
    let end = (start + SCRIPTS_PER_BATCH as usize).min(precached.len());
    let chunk = &precached[start..end];
    let balances = match backend.get_balances_per_script(chunk).await {
        Ok(b) => b,
        Err(e) if e.is_indexing_budget_exhausted() => {
            keystore::set_indexing_progress(coin, *progress)?;
            return Ok(false);
        }
        Err(e) => return Err(e),
    };
    for (script, bal) in chunk.iter().zip(balances) {
        if bal.total_sats() > 0 && !funded_so_far.iter().any(|s| s == script) {
            funded_so_far.push(script.clone());
        }
    }
    if !funded_so_far.is_empty() {
        keystore::set_funded_script_hexes(coin, funded_so_far)?;
    }
    progress.precache_offset = end as u32;
    keystore::set_indexing_progress(coin, *progress)?;
    Ok(end >= precached.len())
}

fn utxo_cache_needs_refresh(coin: CoinId, funded_count: usize) -> bool {
    if funded_count == 0 {
        return false;
    }
    if balance_from_utxo_cache(coin).confirmed_sats == 0 {
        return true;
    }
    LightWalletCache::open(coin)
        .ok()
        .and_then(|c| c.get_meta("utxo_funded_count").ok().flatten())
        .and_then(|s| s.parse::<usize>().ok())
        .map(|n| n != funded_count)
        .unwrap_or(true)
}

async fn maybe_refresh_utxos_from_funded(
    coin: CoinId,
    phrase: &str,
    funded: &[String],
    backend: &dyn ChainBackend,
) -> AppResult<()> {
    if !utxo_cache_needs_refresh(coin, funded.len()) {
        return Ok(());
    }
    backend.clear_initial_indexing_limits().await;
    refresh_utxos(coin, funded, backend, Some(phrase)).await?;
    if let Ok(cache) = LightWalletCache::open(coin) {
        let _ = cache.set_meta("utxo_funded_count", &funded.len().to_string());
    }
    Ok(())
}

async fn run_gap_scan_and_persist(
    coin: CoinId,
    phrase: &str,
    backend: &dyn ChainBackend,
) -> AppResult<()> {
    backend
        .set_initial_indexing_limits(RPC_BUDGET_PER_SYNC, SCRIPTS_PER_BATCH)
        .await;

    let result = run_gap_scan_and_persist_inner(coin, phrase, backend).await;
    backend.clear_initial_indexing_limits().await;
    result
}

async fn run_gap_scan_and_persist_inner(
    coin: CoinId,
    phrase: &str,
    backend: &dyn ChainBackend,
) -> AppResult<()> {
    let mut progress = keystore::indexing_progress(coin)?;
    let mut funded_so_far = keystore::funded_script_hexes(coin)?;

    while !bootstrap_precache_slice(coin, phrase, backend, &mut progress, &mut funded_so_far).await?
    {
        tracing::info!(
            "light wallet {}: precache indexing paused at offset {}",
            coin.as_str(),
            progress.precache_offset
        );
        maybe_refresh_utxos_from_funded(coin, phrase, &funded_so_far, backend).await?;
        return Ok(());
    }

    maybe_refresh_utxos_from_funded(coin, phrase, &funded_so_far, backend).await?;

    let hook = GapScanPersist {
        coin,
        phrase,
        backend,
    };

    const MAX_GAP_SLICES_PER_SYNC: usize = 12;
    let mut scan_complete = false;
    for slice in 0..MAX_GAP_SLICES_PER_SYNC {
        backend
            .set_initial_indexing_limits(RPC_BUDGET_PER_SYNC, SCRIPTS_PER_BATCH)
            .await;

        let progress_before = progress.clone();
        let (_discovered, done) = discover_script_hexes(
            coin,
            phrase,
            None,
            GAP_LIMIT,
            &mut progress,
            backend,
            &mut funded_so_far,
            Some(&hook),
            false,
        )
        .await?;

        keystore::set_indexing_progress(coin, progress)?;
        scan_complete = done;
        maybe_refresh_utxos_from_funded(coin, phrase, &funded_so_far, backend).await?;

        if scan_complete {
            break;
        }
        if progress == progress_before {
            tracing::info!(
                "light wallet {}: gap scan paused (external={}{}, internal={}{})",
                coin.as_str(),
                progress.gap_external,
                if progress.gap_external_done { " done" } else { "" },
                progress.gap_internal,
                if progress.gap_internal_done { " done" } else { "" },
            );
            break;
        }
        if slice + 1 == MAX_GAP_SLICES_PER_SYNC {
            tracing::info!(
                "light wallet {}: gap scan slice budget reached (external={}{}, internal={}{})",
                coin.as_str(),
                progress.gap_external,
                if progress.gap_external_done { " done" } else { "" },
                progress.gap_internal,
                if progress.gap_internal_done { " done" } else { "" },
            );
        }
    }

    if !scan_complete {
        return Ok(());
    }

    backend.clear_initial_indexing_limits().await;
    if funded_so_far.is_empty() {
        let _ = LightWalletCache::open(coin).and_then(|c| c.replace_utxos(&[]));
        mark_precache_probe_complete(coin);
    } else {
        persist_funded_scripts(coin, phrase, backend, &funded_so_far).await?;
        finish_precache_balance_probe(coin, phrase, backend, &mut funded_so_far).await?;
    }
    keystore::mark_address_scan_complete(coin)?;
    Ok(())
}

async fn refresh_utxos(
    coin: CoinId,
    funded_scripts: &[String],
    backend: &dyn ChainBackend,
    phrase: Option<&str>,
) -> AppResult<bool> {
    let electrum_utxos = backend.list_utxos_for_scripts(funded_scripts).await?;
    store_last_electrum_utxo_keys(coin, &electrum_utxos)?;
    let mut utxos = electrum_utxos;
    preserve_local_optimistic_utxos(coin, funded_scripts, &mut utxos)?;
    if let Some(phrase) = phrase {
        enrich_utxo_addresses(coin, phrase, None, &mut utxos)?;
    }
    let mut changed = false;
    if let Ok(cache) = LightWalletCache::open(coin) {
        changed = cache.replace_utxos(&utxos)?;
        if let Ok(tip) = backend.get_tip().await {
            let _ = cache.set_meta("tip_height", &tip.height.to_string());
        }
    }
    for utxo in &utxos {
        clear_local_optimistic_utxo(coin, &utxo.txid, utxo.vout);
    }
    register_funded_scripts_from_utxos(coin, &utxos)?;
    Ok(changed)
}

/// Scripts that hold UTXOs must stay in the funded set so balance refresh keeps querying them.
fn register_funded_scripts_from_utxos(
    coin: CoinId,
    utxos: &[crate::chain::types::Utxo],
) -> AppResult<()> {
    for utxo in utxos {
        if !utxo.script_hex.is_empty() {
            let _ = keystore::register_funded_script_hex(coin, &utxo.script_hex);
        }
    }
    Ok(())
}

/// Probe precached HD scripts not yet marked funded (change / older receive indices).
/// Probe entire precache window per balance cycle when servers can absorb it.
const PRECACHE_PROBE_SCRIPTS_PER_SYNC: usize = 162;

async fn extend_funded_from_precached_balances(
    coin: CoinId,
    phrase: Option<&str>,
    backend: &dyn ChainBackend,
    funded: &mut Vec<String>,
) -> AppResult<()> {
    let precached = keystore::cached_script_hexes(coin)?;
    if precached.is_empty() {
        mark_precache_probe_complete(coin);
        return Ok(());
    }
    let unfunded: Vec<String> = precached
        .into_iter()
        .filter(|s| !funded.iter().any(|f| f.eq_ignore_ascii_case(s)))
        .collect();
    if unfunded.is_empty() {
        mark_precache_probe_complete(coin);
        return Ok(());
    }

    let offset = LightWalletCache::open(coin)
        .ok()
        .and_then(|c| c.get_meta("precache_probe_offset").ok().flatten())
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    let len = unfunded.len();
    let take = PRECACHE_PROBE_SCRIPTS_PER_SYNC.min(len);
    let mut probe: Vec<String> = Vec::with_capacity(take);
    for i in 0..take {
        probe.push(unfunded[(offset + i) % len].clone());
    }
    if let Ok(cache) = LightWalletCache::open(coin) {
        let _ = cache.set_meta("precache_probe_offset", &((offset + take) % len).to_string());
    }

    let batch_size = SCRIPTS_PER_BATCH as usize;
    let mut found_new = false;
    for chunk in probe.chunks(batch_size) {
        let balances = backend.get_balances_per_script(chunk).await?;
        for (script, bal) in chunk.iter().zip(balances) {
            if bal.total_sats() > 0 && !funded.iter().any(|f| f == script) {
                funded.push(script.clone());
                found_new = true;
            }
        }
    }
    if found_new {
        keystore::set_funded_script_hexes(coin, funded)?;
        refresh_utxos(coin, funded, backend, phrase).await?;
    }
    if take >= len {
        mark_precache_probe_complete(coin);
    }
    Ok(())
}

/// After gap scan, probe every precached script so balance matches a full-node wallet.
async fn finish_precache_balance_probe(
    coin: CoinId,
    phrase: &str,
    backend: &dyn ChainBackend,
    funded: &mut Vec<String>,
) -> AppResult<()> {
    let precached = keystore::cached_script_hexes(coin)?;
    let unfunded: Vec<String> = precached
        .into_iter()
        .filter(|s| !funded.iter().any(|f| f.eq_ignore_ascii_case(s)))
        .collect();
    if unfunded.is_empty() {
        mark_precache_probe_complete(coin);
        return Ok(());
    }

    let batch_size = SCRIPTS_PER_BATCH as usize;
    let mut found_new = false;
    for chunk in unfunded.chunks(batch_size) {
        let balances = backend.get_balances_per_script(chunk).await?;
        for (script, bal) in chunk.iter().zip(balances) {
            if bal.total_sats() > 0 && !funded.iter().any(|f| f == script) {
                funded.push(script.clone());
                found_new = true;
            }
        }
    }
    if found_new {
        keystore::set_funded_script_hexes(coin, funded)?;
        refresh_utxos(coin, funded, backend, Some(phrase)).await?;
    }
    mark_precache_probe_complete(coin);
    Ok(())
}

/// Keep locally broadcast 0-conf change until Electrum indexes it.
fn preserve_local_optimistic_utxos(
    coin: CoinId,
    funded_scripts: &[String],
    utxos: &mut Vec<crate::chain::types::Utxo>,
) -> AppResult<()> {
    let funded_set: HashSet<String> = funded_scripts
        .iter()
        .map(|s| s.trim().to_ascii_lowercase())
        .collect();
    if funded_set.is_empty() {
        return Ok(());
    }
    let optimistic = local_optimistic_utxo_keys(coin);
    if optimistic.is_empty() {
        return Ok(());
    }
    let incoming_keys: HashSet<(String, u32)> =
        utxos.iter().map(|u| (u.txid.clone(), u.vout)).collect();
    let cache = LightWalletCache::open(coin)?;
    let cached = cache.list_utxos()?;
    for u in cached {
        if u.height != 0 {
            continue;
        }
        let key = (u.txid.clone(), u.vout);
        if !optimistic.contains(&key) {
            continue;
        }
        if incoming_keys.contains(&key) {
            continue;
        }
        if !funded_set.contains(&u.script_hex.trim().to_ascii_lowercase()) {
            continue;
        }
        utxos.push(u);
    }
    Ok(())
}

const LOCAL_OPTIMISTIC_UTXOS_META: &str = "local_optimistic_utxos";
const LAST_ELECTRUM_UTXO_KEYS_META: &str = "last_electrum_utxo_keys";
const OPTIMISTIC_UTXO_TTL_SECS: u64 = 2 * 60 * 60;

#[derive(serde::Serialize, serde::Deserialize)]
struct LocalOptimisticUtxo {
    txid: String,
    vout: u32,
    added_at: u64,
}

fn utxo_pair_key(txid: &str, vout: u32) -> (String, u32) {
    (txid.to_string(), vout)
}

fn load_local_optimistic_utxos(coin: CoinId) -> AppResult<Vec<LocalOptimisticUtxo>> {
    let cache = LightWalletCache::open(coin)?;
    let Some(raw) = cache.get_meta(LOCAL_OPTIMISTIC_UTXOS_META)? else {
        return Ok(Vec::new());
    };
    serde_json::from_str(&raw)
        .map_err(|e| AppError::other(format!("local optimistic utxo meta: {e}")))
}

fn save_local_optimistic_utxos(coin: CoinId, entries: &[LocalOptimisticUtxo]) -> AppResult<()> {
    let cache = LightWalletCache::open(coin)?;
    let raw = serde_json::to_string(entries)
        .map_err(|e| AppError::other(format!("local optimistic utxo encode: {e}")))?;
    cache.set_meta(LOCAL_OPTIMISTIC_UTXOS_META, &raw)
}

fn prune_expired_optimistic_entries(entries: &mut Vec<LocalOptimisticUtxo>) {
    let now = unix_now_secs();
    entries.retain(|e| now.saturating_sub(e.added_at) <= OPTIMISTIC_UTXO_TTL_SECS);
}

pub fn register_local_optimistic_utxo(coin: CoinId, txid: &str, vout: u32) -> AppResult<()> {
    let mut entries = load_local_optimistic_utxos(coin).unwrap_or_default();
    prune_expired_optimistic_entries(&mut entries);
    let key = utxo_pair_key(txid, vout);
    entries.retain(|e| utxo_pair_key(&e.txid, e.vout) != key);
    entries.push(LocalOptimisticUtxo {
        txid: txid.to_string(),
        vout,
        added_at: unix_now_secs(),
    });
    save_local_optimistic_utxos(coin, &entries)
}

fn clear_local_optimistic_utxo(coin: CoinId, txid: &str, vout: u32) {
    let Ok(mut entries) = load_local_optimistic_utxos(coin) else {
        return;
    };
    let key = utxo_pair_key(txid, vout);
    let before = entries.len();
    entries.retain(|e| utxo_pair_key(&e.txid, e.vout) != key);
    if entries.len() == before {
        return;
    }
    let _ = save_local_optimistic_utxos(coin, &entries);
}

fn local_optimistic_utxo_keys(coin: CoinId) -> HashSet<(String, u32)> {
    let Ok(mut entries) = load_local_optimistic_utxos(coin) else {
        return HashSet::new();
    };
    prune_expired_optimistic_entries(&mut entries);
    entries
        .into_iter()
        .map(|e| utxo_pair_key(&e.txid, e.vout))
        .collect()
}

fn store_last_electrum_utxo_keys(
    coin: CoinId,
    utxos: &[crate::chain::types::Utxo],
) -> AppResult<()> {
    let keys: Vec<String> = utxos
        .iter()
        .map(|u| format!("{}:{}", u.txid, u.vout))
        .collect();
    let raw = serde_json::to_string(&keys)
        .map_err(|e| AppError::other(format!("electrum utxo keys encode: {e}")))?;
    let cache = LightWalletCache::open(coin)?;
    cache.set_meta(LAST_ELECTRUM_UTXO_KEYS_META, &raw)
}

fn has_last_electrum_utxo_snapshot(coin: CoinId) -> bool {
    LightWalletCache::open(coin)
        .ok()
        .and_then(|c| c.get_meta(LAST_ELECTRUM_UTXO_KEYS_META).ok().flatten())
        .is_some()
}

fn last_electrum_utxo_keys(coin: CoinId) -> HashSet<(String, u32)> {
    let Ok(cache) = LightWalletCache::open(coin) else {
        return HashSet::new();
    };
    let Some(raw) = cache
        .get_meta(LAST_ELECTRUM_UTXO_KEYS_META)
        .ok()
        .flatten()
    else {
        return HashSet::new();
    };
    let Ok(keys) = serde_json::from_str::<Vec<String>>(&raw) else {
        return HashSet::new();
    };
    keys.into_iter()
        .filter_map(|k| {
            let (txid, vout) = k.rsplit_once(':')?;
            Some((txid.to_string(), vout.parse().ok()?))
        })
        .collect()
}

pub fn scripts_for_history(coin: CoinId) -> AppResult<Vec<String>> {
    let funded = keystore::funded_script_hexes(coin)?;
    if !funded.is_empty() {
        return Ok(funded);
    }
    keystore::cached_script_hexes(coin)
}

/// P2PKH addresses for scripts the wallet has used (for explorer-indexed history).
pub fn addresses_for_history(coin: CoinId) -> AppResult<Vec<String>> {
    let scripts = scripts_for_history(coin)?;
    if scripts.is_empty() {
        return Ok(Vec::new());
    }

    let mut utxo_addrs: std::collections::HashSet<String> = std::collections::HashSet::new();
    if let Ok(cache) = LightWalletCache::open(coin) {
        for utxo in cache.list_utxos().unwrap_or_default() {
            if !utxo.address.is_empty() {
                utxo_addrs.insert(utxo.address.trim().to_string());
            }
        }
    }

    let mut priority: Vec<String> = Vec::new();
    let mut rest: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    if keystore::is_unlocked(coin)? {
        let phrase = keystore::unlocked_mnemonic(coin, "")?;
        let script_refs: Vec<&str> = scripts.iter().map(String::as_str).collect();
        let map = resolve_addresses_for_script_hexes(coin, &phrase, None, &script_refs)?;
        for addr in map.values() {
            let clean = addr.trim();
            if clean.is_empty() || !seen.insert(clean.to_string()) {
                continue;
            }
            if utxo_addrs.contains(clean) {
                priority.push(clean.to_string());
            } else {
                rest.push(clean.to_string());
            }
        }
    }

    for addr in utxo_addrs {
        if seen.insert(addr.clone()) {
            priority.push(addr);
        }
    }

    rest.sort();
    priority.extend(rest);
    Ok(priority)
}

#[cfg(test)]
mod rescan_cooldown_tests {
    use super::cooldown_remaining_secs;

    #[test]
    fn cooldown_counts_down_from_last_rescan() {
        assert_eq!(cooldown_remaining_secs(1000, 1000, 3600), 3600);
        assert_eq!(cooldown_remaining_secs(1000, 4600, 3600), 0);
        assert_eq!(cooldown_remaining_secs(1000, 2800, 3600), 1800);
    }
}

#[cfg(test)]
mod merge_tx_history_tests {
    use super::merge_tx_history;
    use crate::chain::types::WalletTx;

    fn row(txid: &str, height: i32, blockheight: Option<u32>, category: &str) -> WalletTx {
        WalletTx {
            txid: txid.into(),
            height,
            fee_sats: None,
            category: category.into(),
            amount: -1.0,
            address: Some("addr".into()),
            confirmations: if height > 0 { 1 } else { 0 },
            time: Some(100),
            blockhash: None,
            blockheight,
        }
    }

    #[test]
    fn confirmed_base_is_not_overwritten_by_stale_pending() {
        let base = vec![row("abc", 500, Some(500), "send")];
        let pending = vec![row("abc", 0, None, "send")];
        let merged = merge_tx_history(base, &pending);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].height, 500);
        assert_eq!(merged[0].blockheight, Some(500));
    }

    #[test]
    fn pending_upgrades_to_confirmed_from_electrum() {
        let base = vec![row("abc", 0, None, "send")];
        let confirmed = vec![row("abc", 500, Some(500), "send")];
        let merged = merge_tx_history(base, &confirmed);
        assert_eq!(merged[0].height, 500);
        assert_eq!(merged[0].blockheight, Some(500));
    }
}
