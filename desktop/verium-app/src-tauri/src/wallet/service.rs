//! Wallet service facade routing full-node vs light backends.

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
use crate::wallet::sync::{balance_from_utxo_cache, scripts_for_history, sync_light_wallet};
use crate::wallet::utxo_selector::{plan_send_utxos, DEFAULT_TX_FEE_COINS_PER_KB, fee_for_rate};

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
    let funded_scripts = keystore::funded_script_hexes(coin).unwrap_or_default();
    let should_background_sync =
        light_syncing || (scan_complete && !funded_scripts.is_empty());

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

    Ok(Some(json!({
        "walletname": format!("{}-light", coin.as_str()),
        "balance": sats_to_coins(bal.confirmed_sats),
        "unconfirmed_balance": sats_to_coins(bal.unconfirmed_sats),
        "immature_balance": sats_to_coins(bal.immature_sats),
        "txcount": 0,
        "keypoolsize": 0,
        "unlocked_until": unlocked_until,
        "walletversion": 1,
        "paytxfee": prefs.tx_fee_rate_vrm_per_kb.unwrap_or(0.0001),
        "hdseedid": "light",
        "private_keys_enabled": signing_ready,
        "light_wallet": true,
        "light_syncing": light_syncing,
    })))
}

pub async fn list_transactions(
    state: &AppState,
    coin: CoinId,
    count: usize,
    passphrase: Option<&str>,
) -> AppResult<Vec<Value>> {
    let prefs = prefs::load().await?;
    if !prefs::wallet_mode_for(&prefs, coin).is_light() {
        return Ok(vec![]);
    }
    let _ = passphrase;
    let backend = resolve_backend(state, coin).await?;
    let scripts = scripts_for_history(coin)?;
    if scripts.is_empty() {
        return Ok(vec![]);
    }
    let txs = backend.get_history_for_scripts(&scripts, count).await?;
    Ok(txs
        .into_iter()
        .map(|t| {
            json!({
                "txid": t.txid,
                "category": t.category,
                "amount": t.amount,
                "confirmations": t.confirmations,
                "address": t.address,
                "blockheight": t.blockheight,
                "blockhash": t.blockhash,
                "time": t.time,
                "fee": t.fee_sats.map(|f| sats_to_coins(f)),
            })
        })
        .collect())
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
    let phrase = keystore::decrypt_mnemonic_for_send(coin, passphrase)?;
    let utxos = fetch_utxos_with_addresses(state, coin, &phrase).await?;
    let backend = resolve_backend(state, coin).await?;
    let amount_sats = coins_to_sats(amount);
    let rate = fee_rate
        .unwrap_or(
            prefs
                .tx_fee_rate_vrm_per_kb
                .unwrap_or(DEFAULT_TX_FEE_COINS_PER_KB),
        );
    let (selected, fee_sats) = plan_send_utxos(&utxos, amount_sats, rate, 2)?;
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
    backend.broadcast_tx(&signed.hex).await
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
    let phrase = keystore::decrypt_mnemonic_for_send(coin, passphrase)?;
    let all_utxos = fetch_utxos_with_addresses(state, coin, &phrase).await?;
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
    let fee_sats = fee_for_rate(rate, selected.len(), output_pairs.len() + 1);
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
    let backend = resolve_backend(state, coin).await?;
    backend.broadcast_tx(&signed.hex).await
}

pub async fn light_server_status(state: &AppState, coin: CoinId) -> AppResult<Option<LightServerStatus>> {
    let prefs = prefs::load().await?;
    if !prefs::wallet_mode_for(&prefs, coin).is_light() {
        return Ok(None);
    }
    let backend = resolve_backend(state, coin).await?;
    Ok(backend.light_server_status().await)
}
