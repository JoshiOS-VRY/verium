//! Wallet service facade routing full-node vs light backends.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;
use serde_json::{json, Value};

use crate::chain::types::*;
use crate::coin_profile::CoinId;
use crate::error::{AppError, AppResult};
use crate::prefs::{self, UserPreferences};
use crate::state::AppState;
use crate::wallet::backend::resolve_backend;
use crate::wallet::cache::LightWalletCache;
use crate::wallet::hd::{
    coins_to_sats, derive_address_at, derive_change_address_at, enrich_utxo_addresses,
    sats_to_coins, uses_core_hd_paths,
};
use crate::wallet::keystore;
use crate::wallet::mode::WalletMode;
use crate::wallet::signer;
use crate::wallet::explorer_history::fetch_wallet_history_from_explorer;
use crate::wallet::sync::{
    addresses_for_history, balance_from_utxo_cache, sync_light_wallet,
};
use crate::wallet::utxo_selector::{
    plan_send_utxos, replan_fee_for_selected, DEFAULT_TX_FEE_COINS_PER_KB,
};

fn parent_txid_variants(txid: &str) -> Vec<String> {
    let trimmed = txid.trim();
    let mut variants = vec![trimmed.to_string()];
    if let Ok(alt) = crate::wallet::vericonomy_tx::reverse_display_txid_hex(trimmed) {
        if !variants.iter().any(|v| v.eq_ignore_ascii_case(&alt)) {
            variants.push(alt);
        }
    }
    variants
}

async fn fetch_parent_tx_bytes(
    backend: &dyn crate::chain::ChainBackend,
    txid: &str,
) -> AppResult<Vec<u8>> {
    let mut last_err: Option<AppError> = None;
    for variant in parent_txid_variants(txid) {
        match backend.get_raw_tx_hex(&variant).await {
            Ok(hex) => {
                let bytes = hex::decode(&hex)
                    .map_err(|e| AppError::other(format!("parent tx hex decode: {e}")))?;
                return Ok(bytes);
            }
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.unwrap_or_else(|| AppError::other("failed to fetch parent transaction")))
}

/// Fetch each parent tx from Electrum, normalize txid/script/value, and verify the spend.
async fn prepare_utxos_for_signing(
    backend: &dyn crate::chain::ChainBackend,
    utxos: &mut [Utxo],
) -> AppResult<()> {
    for utxo in utxos.iter_mut() {
        let bytes = fetch_parent_tx_bytes(backend, &utxo.txid).await?;
        let display_txid = crate::wallet::vericonomy_tx::display_txid_from_raw(&bytes);
        let wire_txid = crate::wallet::vericonomy_tx::wire_txid_from_raw(&bytes);
        let parsed = crate::wallet::vericonomy_tx::parse_display_txid(&display_txid)?;
        if parsed != wire_txid {
            return Err(AppError::other(format!(
                "parent txid endian mismatch for {}",
                utxo.txid
            )));
        }

        let parent = crate::wallet::vericonomy_tx::decode_verium_tx(&bytes)
            .map_err(|e| AppError::other(format!("parent tx decode: {e}")))?;
        let out = parent
            .outputs
            .get(utxo.vout as usize)
            .ok_or_else(|| AppError::other(format!("parent tx missing vout {}", utxo.vout)))?;
        let on_chain_script = hex::encode(out.script_pubkey.as_bytes());
        let on_chain_value = out.value.to_sat() as i64;

        if on_chain_value != utxo.value_sats {
            tracing::warn!(
                "UTXO {}:{} value stale (cache {} vs chain {}); using chain value",
                utxo.txid,
                utxo.vout,
                utxo.value_sats,
                on_chain_value
            );
            utxo.value_sats = on_chain_value;
        }

        if !display_txid.eq_ignore_ascii_case(&utxo.txid) {
            tracing::info!(
                "UTXO {}:{} txid normalized {} -> {}",
                utxo.txid,
                utxo.vout,
                utxo.txid,
                display_txid
            );
        }

        if !crate::wallet::verify::script_pays_to(&on_chain_script, &utxo.script_hex) {
            return Err(AppError::other(format!(
                "UTXO {}:{} on-chain script does not match wallet script",
                utxo.txid,
                utxo.vout
            )));
        }

        utxo.txid = display_txid;
        utxo.script_hex = on_chain_script;
    }
    Ok(())
}

pub async fn fetch_utxos_with_addresses(
    state: &AppState,
    coin: CoinId,
    phrase: &str,
) -> AppResult<Vec<Utxo>> {
    let cached = LightWalletCache::open(coin)
        .ok()
        .and_then(|c| c.list_utxos().ok())
        .unwrap_or_default();
    let scan_incomplete = keystore::needs_full_address_scan(coin).unwrap_or(false);

    let mut utxos = if scan_incomplete && !cached.is_empty() {
        let sync_state = state.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = sync_light_wallet(&sync_state, coin).await {
                tracing::warn!("background light wallet sync during send: {e}");
            }
        });
        cached
    } else {
        sync_light_wallet(state, coin).await?;
        LightWalletCache::open(coin)?
            .list_utxos()
            .map_err(|e| AppError::other(format!("utxo cache read: {e}")))?
    };

    enrich_utxo_addresses(coin, phrase, None, &mut utxos)?;
    if let Some(missing) = utxos.iter().find(|u| u.address.is_empty() && !u.script_hex.is_empty()) {
        return Err(AppError::other(format!(
            "could not resolve signing address for utxo {}:{}",
            missing.txid, missing.vout
        )));
    }
    Ok(utxos)
}

/// UTXO set used for signing: always sync from Electrum first so txid/script/value match chain.
async fn fetch_utxos_for_send(
    state: &AppState,
    coin: CoinId,
    phrase: &str,
) -> AppResult<Vec<Utxo>> {
    sync_light_wallet(state, coin).await?;
    let mut utxos = LightWalletCache::open(coin)?
        .list_utxos()
        .map_err(|e| AppError::other(format!("utxo cache read: {e}")))?;
    enrich_utxo_addresses(coin, phrase, None, &mut utxos)?;
    if let Some(missing) = utxos.iter().find(|u| u.address.is_empty() && !u.script_hex.is_empty()) {
        return Err(AppError::other(format!(
            "could not resolve signing address for utxo {}:{} (wait for address scan to finish)",
            missing.txid, missing.vout
        )));
    }
    Ok(utxos)
}

/// Minimum gap between steady-state (scan-complete) light-wallet UTXO refetches.
/// `get_wallet_info` is polled every ~30s; without this throttle each poll would
/// trigger a full Electrum UTXO refetch + SQLite table rewrite. Initial-scan
/// syncs (`light_syncing`) are NOT throttled so they keep making progress.
const STEADY_SYNC_MIN_INTERVAL: Duration = Duration::from_secs(5);
/// Minimum gap between gap-scan indexing slices (Electrum RPC budget per slice).
const INDEXING_SYNC_MIN_INTERVAL: Duration = Duration::from_secs(5);

static LAST_STEADY_SYNC: Lazy<Mutex<HashMap<CoinId, Instant>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static LAST_INDEXING_SYNC: Lazy<Mutex<HashMap<CoinId, Instant>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Returns true if a steady-state background sync should run now (and records the
/// attempt). Throttled per coin to `STEADY_SYNC_MIN_INTERVAL`.
fn steady_sync_due(coin: CoinId) -> bool {
    let mut map = match LAST_STEADY_SYNC.lock() {
        Ok(m) => m,
        Err(_) => return true,
    };
    let now = Instant::now();
    match map.get(&coin) {
        Some(last) if now.duration_since(*last) < STEADY_SYNC_MIN_INTERVAL => false,
        _ => {
            map.insert(coin, now);
            true
        }
    }
}

fn indexing_sync_due(coin: CoinId) -> bool {
    let mut map = match LAST_INDEXING_SYNC.lock() {
        Ok(m) => m,
        Err(_) => return true,
    };
    let now = Instant::now();
    match map.get(&coin) {
        Some(last) if now.duration_since(*last) < INDEXING_SYNC_MIN_INTERVAL => false,
        _ => {
            map.insert(coin, now);
            true
        }
    }
}

/// Clear sync throttles so the next poll resyncs immediately (unlock/import/rescan).
pub fn reset_steady_sync_throttle(coin: CoinId) {
    if let Ok(mut map) = LAST_STEADY_SYNC.lock() {
        map.remove(&coin);
    }
    if let Ok(mut map) = LAST_INDEXING_SYNC.lock() {
        map.remove(&coin);
    }
    crate::wallet::sync::reset_pending_refresh_throttle(coin);
    crate::wallet::sync::reset_balance_refresh_throttle(coin);
}

/// Last Electrum tip height we reacted to per coin (for event-driven sync).
static LAST_TIP_SYNCED: Lazy<Mutex<HashMap<CoinId, u32>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Event-driven steady-state sync: when the light server's chain tip advances,
/// refresh the wallet (deduped + rate-limited by the steady-sync throttle)
/// instead of waiting out the fixed fallback timer. Only fires once initial
/// address scanning is complete; the initial scan is driven by `get_wallet_info`.
fn maybe_sync_on_tip_advance(state: &AppState, coin: CoinId, tip: u32) {
    if keystore::needs_full_address_scan(coin).unwrap_or(true) {
        return;
    }
    let advanced = {
        match LAST_TIP_SYNCED.lock() {
            Ok(mut map) => {
                let advanced = map.get(&coin).map(|&last| tip > last).unwrap_or(true);
                if advanced {
                    map.insert(coin, tip);
                }
                advanced
            }
            Err(_) => false,
        }
    };
    if !advanced {
        return;
    }
    // Merge Electrum 0-conf / new-block txs immediately (incoming notify latency).
    let pending_state = state.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = crate::wallet::sync::refresh_light_wallet_pending(&pending_state, coin).await
        {
            tracing::debug!("tip-advance pending refresh failed for {}: {e}", coin.as_str());
        }
    });
    // Rate-limit full UTXO + explorer history refresh.
    if !steady_sync_due(coin) {
        return;
    }
    let funded = keystore::funded_script_hexes(coin).unwrap_or_default();
    if funded.is_empty() {
        return;
    }
    let sync_state = state.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = sync_light_wallet(&sync_state, coin).await {
            tracing::debug!("tip-advance light sync failed for {}: {e}", coin.as_str());
        }
    });
}

pub fn wallet_mode_for(prefs: &UserPreferences, coin: CoinId) -> WalletMode {
    prefs::wallet_mode_for(prefs, coin)
}

pub fn is_light_mode(prefs: &UserPreferences, coin: CoinId) -> bool {
    prefs::wallet_mode_for(prefs, coin).is_light()
}

pub async fn get_wallet_info_json(
    state: &AppState,
    coin: CoinId,
    _passphrase: Option<&str>,
) -> AppResult<Option<Value>> {
    let prefs = prefs::load().await?;
    if !prefs::wallet_mode_for(&prefs, coin).is_light() || !keystore::wallet_exists(coin)? {
        return Ok(None);
    }

    let signing_ready = keystore::signing_session_active(coin);
    let unlock_timer_active = keystore::is_unlocked(coin).unwrap_or(false);
    let session_unlocked = unlock_timer_active && signing_ready;
    let scan_complete = !keystore::needs_full_address_scan(coin).unwrap_or(true);
    let light_syncing = session_unlocked && !scan_complete;
    let sync_in_flight = crate::wallet::sync::is_light_wallet_sync_in_flight(coin).await;
    let probe_complete = crate::wallet::sync::is_precache_probe_complete(coin);
    let light_balance_syncing = session_unlocked && (light_syncing || sync_in_flight);
    let light_balance_ready =
        session_unlocked && scan_complete && probe_complete && !sync_in_flight;
    // Gap-scan syncs run from getwalletinfo while addresses are still being discovered.
    // Steady-state balance refresh is driven by the foreground balance poll (`light_wallet_refresh_balance`).
    let should_background_sync = light_syncing && indexing_sync_due(coin);

    if should_background_sync {
        let sync_state = state.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = sync_light_wallet(&sync_state, coin).await {
                tracing::error!("light wallet sync failed for {}: {e}", coin.as_str());
            }
        });
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let unlocked_until = if session_unlocked {
        keystore::load_keystore()?
            .unlocked_until_by_coin
            .get(coin.as_str())
            .copied()
            .unwrap_or(now)
    } else {
        0
    };

    let bal = balance_from_utxo_cache(coin);
    // Spendable includes 0-conf change outputs so a 1 VRM send does not drop
    // displayed balance by the full input amount (Core splits confirmed vs pending).
    let available_sats = bal.confirmed_sats + bal.unconfirmed_sats;
    let txcount = LightWalletCache::open(coin)
        .and_then(|c| c.tx_history_count())
        .unwrap_or(0);

    Ok(Some(json!({
        "walletname": format!("{}-light", coin.as_str()),
        "balance": sats_to_coins(available_sats),
        "confirmed_balance": sats_to_coins(bal.confirmed_sats),
        "unconfirmed_balance": sats_to_coins(bal.unconfirmed_sats),
        "immature_balance": sats_to_coins(bal.immature_sats),
        "txcount": txcount,
        "keypoolsize": 0,
        "unlocked_until": unlocked_until,
        "walletversion": 1,
        "paytxfee": prefs.tx_fee_rate_vrm_per_kb.unwrap_or(0.0001),
        "hdseedid": "light",
        "private_keys_enabled": signing_ready,
        "light_wallet": true,
        "light_syncing": light_syncing,
        "light_balance_syncing": light_balance_syncing,
        "light_balance_ready": light_balance_ready,
    })))
}

pub async fn list_transactions(
    _state: &AppState,
    coin: CoinId,
    count: usize,
    passphrase: Option<&str>,
) -> AppResult<Vec<Value>> {
    let prefs = prefs::load().await?;
    if !prefs::wallet_mode_for(&prefs, coin).is_light() {
        return Ok(vec![]);
    }
    let _ = passphrase;

    if let Ok(cache) = LightWalletCache::open(coin) {
        let rows = cache.list_tx_history(count).unwrap_or_default();
        if !rows.is_empty() {
            let tip = cached_tip_height(&cache);
            let enriched = rows.iter().any(|t| t.time.is_some() && t.time.unwrap_or(0) > 0);
            if enriched {
                return Ok(rows.iter().map(|t| wallet_tx_to_json(t, tip)).collect());
            }

            // Stale cache (txid-only rows): try explorer once, never Electrum on read.
            let explorer_tip = tip;
            if let Ok(addresses) = addresses_for_history(coin) {
                if let Ok(history) = fetch_wallet_history_from_explorer(
                    coin,
                    &addresses,
                    count,
                    explorer_tip,
                )
                .await
                {
                    if !history.is_empty() {
                        let _ = cache.replace_tx_history(&history);
                        return Ok(history.iter().map(|t| wallet_tx_to_json(t, explorer_tip)).collect());
                    }
                }
            }
            return Ok(rows.iter().map(|t| wallet_tx_to_json(t, tip)).collect());
        }
    }

    // Empty cache: explorer only (sync also refreshes history in the background).
    let tip = LightWalletCache::open(coin)
        .ok()
        .and_then(|c| cached_tip_height(&c));
    if let Ok(addresses) = addresses_for_history(coin) {
        if let Ok(history) = fetch_wallet_history_from_explorer(coin, &addresses, count, tip).await
        {
            if !history.is_empty() {
                if let Ok(cache) = LightWalletCache::open(coin) {
                    let _ = cache.replace_tx_history(&history);
                    if let Some(height) = tip {
                        let _ = cache.set_meta("tip_height", &height.to_string());
                    }
                }
                return Ok(history.iter().map(|t| wallet_tx_to_json(t, tip)).collect());
            }
        }
    }
    Ok(vec![])
}

fn cached_tip_height(cache: &LightWalletCache) -> Option<u32> {
    cache
        .get_meta("tip_height")
        .ok()
        .flatten()
        .and_then(|s| s.parse::<u32>().ok())
}

/// Serialize a cached `WalletTx` to the wallet JSON shape, recomputing
/// confirmations from the current chain tip so cached rows stay accurate
/// between history refreshes.
fn wallet_tx_to_json(t: &WalletTx, tip: Option<u32>) -> Value {
    let confirmations = match (tip, t.blockheight) {
        (Some(tip), Some(bh)) if tip >= bh => (tip - bh + 1) as i64,
        (_, Some(_)) => t.confirmations as i64,
        _ => 0,
    };
    let time = t.time.unwrap_or(0);
    json!({
        "txid": t.txid,
        "category": t.category,
        "amount": t.amount,
        "confirmations": confirmations,
        "address": t.address,
        "blockheight": t.blockheight,
        "blockhash": t.blockhash,
        "time": time,
        "timereceived": time,
        "fee": t.fee_sats.map(sats_to_coins),
    })
}

pub async fn get_new_address(
    _state: &AppState,
    coin: CoinId,
    passphrase: Option<&str>,
) -> AppResult<String> {
    let prefs = prefs::load().await?;
    if !prefs::wallet_mode_for(&prefs, coin).is_light() {
        return Err(AppError::other("not in light wallet mode"));
    }
    if !keystore::is_unlocked(coin)? && passphrase.is_none() {
        return Err(AppError::other("wallet is locked"));
    }
    let phrase = keystore::unlocked_mnemonic(coin, passphrase.unwrap_or(""))?;
    let index = keystore::bump_receive_index(coin)?;
    derive_address_at(coin, &phrase, None, index)
}

pub async fn send_to_address(
    state: &AppState,
    coin: CoinId,
    address: &str,
    amount: f64,
    fee_rate: Option<f64>,
    passphrase: &str,
) -> AppResult<String> {
    let prefs = prefs::load().await?;
    if !prefs::wallet_mode_for(&prefs, coin).is_light() {
        return Err(AppError::other("not in light wallet mode"));
    }
    let phrase = keystore::unlocked_mnemonic(coin, passphrase)?;
    let utxos = fetch_utxos_for_send(state, coin, &phrase).await?;
    let backend = resolve_backend(state, coin).await?;
    let amount_sats = coins_to_sats(amount);
    let rate = fee_rate
        .unwrap_or(
            prefs
                .tx_fee_rate_vrm_per_kb
                .unwrap_or(DEFAULT_TX_FEE_COINS_PER_KB),
        );
    let (mut selected, _initial_fee) = plan_send_utxos(&utxos, amount_sats, rate, 1)?;
    prepare_utxos_for_signing(backend.as_ref(), &mut selected).await?;
    let fee_sats = replan_fee_for_selected(&selected, amount_sats, rate, 1)?;
    let change_idx = keystore::peek_receive_index(coin)?.saturating_sub(1);
    let change_addr = if uses_core_hd_paths(coin, &phrase) {
        derive_change_address_at(coin, &phrase, None, change_idx)?
    } else {
        derive_address_at(coin, &phrase, None, change_idx)?
    };
    let signed = signer::sign_transaction(
        coin,
        &phrase,
        None,
        &selected,
        &[(address.to_string(), amount_sats)],
        &change_addr,
        fee_sats,
    )?;
    crate::wallet::verify::verify_send_outputs(
        coin,
        &signed.hex,
        &[(address.to_string(), amount_sats)],
    )?;
    crate::wallet::verify::verify_signed_p2pkh_inputs(&signed.hex, &selected)?;
    let txid = backend.broadcast_tx(&signed.hex).await?;
    for utxo in &selected {
        if let Err(e) = keystore::register_funded_address(coin, &utxo.address) {
            tracing::warn!(
                "register input address after send failed for {}: {e}",
                coin.as_str()
            );
        }
    }
    if let Err(e) = keystore::register_funded_address(coin, &change_addr) {
        tracing::warn!(
            "register change address after send failed for {}: {e}",
            coin.as_str()
        );
    }
    let input_sum: i64 = selected.iter().map(|u| u.value_sats).sum();
    let change_sats = input_sum - amount_sats - fee_sats;
    if let Err(e) = crate::wallet::sync::apply_local_send_cache_update(
        coin,
        &selected,
        &txid,
        &signed.hex,
        change_sats,
        &change_addr,
    ) {
        tracing::warn!("post-send local cache update failed for {}: {e}", coin.as_str());
    }
    reset_steady_sync_throttle(coin);
    crate::wallet::sync::reset_balance_refresh_throttle(coin);
    Ok(txid)
}

pub async fn list_unspent_json(
    state: &AppState,
    coin: CoinId,
    minconf: u32,
    passphrase: &str,
) -> AppResult<Vec<Value>> {
    let prefs = prefs::load().await?;
    if !prefs::wallet_mode_for(&prefs, coin).is_light() {
        return Err(AppError::other("not in light wallet mode"));
    }
    let phrase = keystore::unlocked_mnemonic(coin, passphrase)?;
    let utxos = fetch_utxos_with_addresses(state, coin, &phrase).await?;
    Ok(utxos
        .into_iter()
        .filter(|u| u.confirmations >= minconf)
        .map(|u| {
            json!({
                "txid": u.txid,
                "vout": u.vout,
                "address": u.address,
                "amount": sats_to_coins(u.value_sats),
                "confirmations": u.confirmations,
                "spendable": true,
                "solvable": true,
                "scriptPubKey": u.script_hex,
            })
        })
        .collect())
}

pub async fn send_with_inputs(
    state: &AppState,
    coin: CoinId,
    inputs: &[Value],
    outputs: &serde_json::Map<String, Value>,
    change_address: Option<&str>,
    fee_rate: Option<f64>,
    passphrase: &str,
) -> AppResult<String> {
    let prefs = prefs::load().await?;
    if !prefs::wallet_mode_for(&prefs, coin).is_light() {
        return Err(AppError::other("not in light wallet mode"));
    }
    let phrase = keystore::unlocked_mnemonic(coin, passphrase)?;
    let all_utxos = fetch_utxos_for_send(state, coin, &phrase).await?;
    let mut selected = Vec::new();
    for input in inputs {
        let txid = input
            .get("txid")
            .and_then(Value::as_str)
            .ok_or_else(|| AppError::other("input missing txid"))?;
        let vout = input
            .get("vout")
            .and_then(Value::as_u64)
            .ok_or_else(|| AppError::other("input missing vout"))? as u32;
        let utxo = all_utxos
            .iter()
            .find(|u| u.txid == txid && u.vout == vout)
            .cloned()
            .ok_or_else(|| AppError::other(format!("unknown input {txid}:{vout}")))?;
        selected.push(utxo);
    }
    let mut output_pairs = Vec::new();
    for (addr, amount_v) in outputs {
        let amount = amount_v
            .as_f64()
            .ok_or_else(|| AppError::other(format!("invalid amount for {addr}")))?;
        output_pairs.push((addr.clone(), coins_to_sats(amount)));
    }
    let rate = fee_rate.unwrap_or(
        prefs
            .tx_fee_rate_vrm_per_kb
            .unwrap_or(DEFAULT_TX_FEE_COINS_PER_KB),
    );
    let change_addr = match change_address.filter(|s| !s.is_empty()) {
        Some(a) => a.to_string(),
        None => {
            let change_idx = keystore::peek_receive_index(coin)?.saturating_sub(1);
            if uses_core_hd_paths(coin, &phrase) {
                derive_change_address_at(coin, &phrase, None, change_idx)?
            } else {
                derive_address_at(coin, &phrase, None, change_idx)?
            }
        }
    };
    let backend = resolve_backend(state, coin).await?;
    prepare_utxos_for_signing(backend.as_ref(), &mut selected).await?;
    let total_out: i64 = output_pairs.iter().map(|(_, v)| *v).sum();
    let fee_sats = replan_fee_for_selected(&selected, total_out, rate, output_pairs.len())?;
    let signed = signer::sign_transaction(
        coin,
        &phrase,
        None,
        &selected,
        &output_pairs,
        &change_addr,
        fee_sats,
    )?;
    crate::wallet::verify::verify_send_outputs(coin, &signed.hex, &output_pairs)?;
    crate::wallet::verify::verify_signed_p2pkh_inputs(&signed.hex, &selected)?;
    let txid = backend.broadcast_tx(&signed.hex).await?;
    for utxo in &selected {
        if let Err(e) = keystore::register_funded_address(coin, &utxo.address) {
            tracing::warn!(
                "register input address after send failed for {}: {e}",
                coin.as_str()
            );
        }
    }
    if let Err(e) = keystore::register_funded_address(coin, &change_addr) {
        tracing::warn!(
            "register change address after send failed for {}: {e}",
            coin.as_str()
        );
    }
    let input_sum: i64 = selected.iter().map(|u| u.value_sats).sum();
    let change_sats = input_sum - total_out - fee_sats;
    if let Err(e) = crate::wallet::sync::apply_local_send_cache_update(
        coin,
        &selected,
        &txid,
        &signed.hex,
        change_sats,
        &change_addr,
    ) {
        tracing::warn!("post-send local cache update failed for {}: {e}", coin.as_str());
    }
    reset_steady_sync_throttle(coin);
    crate::wallet::sync::reset_balance_refresh_throttle(coin);
    Ok(txid)
}

pub async fn light_server_status(state: &AppState, coin: CoinId) -> AppResult<Option<LightServerStatus>> {
    let prefs = prefs::load().await?;
    if !prefs::wallet_mode_for(&prefs, coin).is_light() {
        return Ok(None);
    }
    let backend = resolve_backend(state, coin).await?;
    let status = backend.light_server_status().await;
    if let Some(s) = &status {
        if let Some(tip) = s.tip_height {
            maybe_sync_on_tip_advance(state, coin, tip);
        }
    }
    Ok(status)
}
