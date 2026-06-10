//! Light-wallet chain sync: gap scan → funded scripts → UTXO set → balance.

use std::collections::HashSet;
use std::time::Duration;

use once_cell::sync::Lazy;
use tokio::sync::Mutex;

use crate::chain::electrum::indexing::{RPC_BUDGET_PER_SYNC, SCRIPTS_PER_BATCH};
use crate::chain::types::WalletBalance;
use crate::chain::ChainBackend;
use crate::coin_profile::CoinId;
use crate::error::AppResult;
use crate::state::AppState;
use async_trait::async_trait;

use crate::wallet::cache::LightWalletCache;
use crate::wallet::gap_scan_hook::GapScanHook;
use crate::wallet::hd::{discover_script_hexes, enrich_utxo_addresses};
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
const SYNC_TIMEOUT: Duration = Duration::from_secs(120);

static SYNC_IN_FLIGHT: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));

pub fn balance_from_utxo_cache(coin: CoinId) -> WalletBalance {
    let confirmed = LightWalletCache::open(coin)
        .ok()
        .and_then(|c| c.sum_utxo_values().ok())
        .unwrap_or(0);
    WalletBalance {
        confirmed_sats: confirmed,
        unconfirmed_sats: 0,
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

    let funded = keystore::funded_script_hexes(coin)?;
    if funded.is_empty() {
        return Ok(());
    }
    refresh_utxos(coin, &funded, backend.as_ref(), None).await?;
    // Refresh the persisted tx-history cache on the same (tip-driven) cadence so
    // list_transactions reads from local SQLite instead of issuing N Electrum
    // get_history calls on every UI poll. Best-effort: never fails the sync.
    refresh_tx_history_cache(coin, &funded, backend.as_ref()).await;
    Ok(())
}

/// Number of history rows kept in the local cache (matches UI list cap).
pub const TX_HISTORY_CACHE_LIMIT: usize = 500;

async fn refresh_tx_history_cache(coin: CoinId, funded: &[String], backend: &dyn ChainBackend) {
    if funded.is_empty() {
        return;
    }
    match backend
        .get_history_for_scripts(funded, TX_HISTORY_CACHE_LIMIT)
        .await
    {
        Ok(history) => {
            if let Ok(cache) = LightWalletCache::open(coin) {
                if let Err(e) = cache.replace_tx_history(&history) {
                    tracing::debug!("tx history cache write failed for {}: {e}", coin.as_str());
                }
            }
        }
        Err(e) => {
            tracing::debug!("tx history refresh failed for {}: {e}", coin.as_str());
        }
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
    refresh_utxos(coin, funded, backend, Some(phrase)).await
}

async fn bootstrap_precache_slice(
    coin: CoinId,
    _phrase: &str,
    backend: &dyn ChainBackend,
    progress: &mut keystore::IndexingProgress,
    funded_so_far: &mut Vec<String>,
) -> AppResult<bool> {
    let precached = keystore::cached_script_hexes(coin)?;
    if precached.is_empty() {
        return Ok(true);
    }
    if progress.precache_offset as usize >= precached.len() {
        return Ok(true);
    }

    loop {
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
        if end >= precached.len() {
            return Ok(true);
        }
    }
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
    backend
        .set_initial_indexing_limits(RPC_BUDGET_PER_SYNC, SCRIPTS_PER_BATCH)
        .await;

    let hook = GapScanPersist {
        coin,
        phrase,
        backend,
    };

    let (_discovered, scan_complete) = discover_script_hexes(
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

    if !scan_complete {
        tracing::info!(
            "light wallet {}: gap scan indexing slice complete (external={}{}, internal={}{})",
            coin.as_str(),
            progress.gap_external,
            if progress.gap_external_done { " done" } else { "" },
            progress.gap_internal,
            if progress.gap_internal_done { " done" } else { "" },
        );
        maybe_refresh_utxos_from_funded(coin, phrase, &funded_so_far, backend).await?;
        return Ok(());
    }

    backend.clear_initial_indexing_limits().await;
    if funded_so_far.is_empty() {
        let _ = LightWalletCache::open(coin).and_then(|c| c.replace_utxos(&[]));
    } else {
        persist_funded_scripts(coin, phrase, backend, &funded_so_far).await?;
    }
    keystore::mark_address_scan_complete(coin)
}

async fn refresh_utxos(
    coin: CoinId,
    funded_scripts: &[String],
    backend: &dyn ChainBackend,
    phrase: Option<&str>,
) -> AppResult<()> {
    let mut utxos = backend.list_utxos_for_scripts(funded_scripts).await?;
    if let Some(phrase) = phrase {
        enrich_utxo_addresses(coin, phrase, None, &mut utxos)?;
    }
    if let Ok(cache) = LightWalletCache::open(coin) {
        cache.replace_utxos(&utxos)?;
        if let Ok(tip) = backend.get_tip().await {
            let _ = cache.set_meta("tip_height", &tip.height.to_string());
        }
    }
    Ok(())
}

pub fn scripts_for_history(coin: CoinId) -> AppResult<Vec<String>> {
    let funded = keystore::funded_script_hexes(coin)?;
    if !funded.is_empty() {
        return Ok(funded);
    }
    keystore::cached_script_hexes(coin)
}
