//! PSBT construction for light wallet (local UTXO fetch, no daemon RPC).

use serde_json::{Map, Value};

use crate::coin_profile::CoinId;
use crate::error::{AppError, AppResult};
use crate::hardware_wallet::PsbtSendResult;
use crate::state::AppState;
use crate::wallet::hd::{coins_to_sats, derive_address_at, derive_change_address_at, uses_core_hd_paths};
use crate::wallet::keystore;
use crate::wallet::service;
use crate::wallet::signer;
use crate::wallet::utxo_selector::{plan_send_utxos, DEFAULT_TX_FEE_COINS_PER_KB};

/// Build an unsigned transaction (hex, base64-wrapped) for hardware signing.
pub async fn build_hw_psbt(
    state: &AppState,
    coin: CoinId,
    outputs: Map<String, Value>,
    fee_rate: Option<f64>,
    passphrase: &str,
) -> AppResult<PsbtSendResult> {
    let prefs = crate::prefs::load().await?;
    if !prefs.wallet_mode.is_light() {
        return Err(AppError::other("not in light wallet mode"));
    }
    let phrase = keystore::unlocked_mnemonic(coin, passphrase)?;
    let utxos = service::fetch_utxos_with_addresses(state, coin, &phrase).await?;
    let mut output_pairs = Vec::new();
    let mut total_out = 0i64;
    for (addr, amount_v) in &outputs {
        let amount = amount_v
            .as_f64()
            .ok_or_else(|| AppError::other(format!("invalid amount for {addr}")))?;
        let sats = coins_to_sats(amount);
        total_out += sats;
        output_pairs.push((addr.clone(), sats));
    }
    let rate = fee_rate.unwrap_or(
        prefs
            .tx_fee_rate_vrm_per_kb
            .unwrap_or(DEFAULT_TX_FEE_COINS_PER_KB),
    );
    let (selected, fee_sats) =
        plan_send_utxos(&utxos, total_out, rate, output_pairs.len() + 1)?;
    let change_idx = keystore::peek_receive_index(coin)?.saturating_sub(1);
    let change_addr = if uses_core_hd_paths(coin, &phrase) {
        derive_change_address_at(coin, &phrase, None, change_idx)?
    } else {
        derive_address_at(coin, &phrase, None, change_idx)?
    };
    let unsigned_hex = signer::build_unsigned_hex(
        coin,
        &phrase,
        None,
        &selected,
        &output_pairs,
        &change_addr,
        fee_sats,
    )?;
    Ok(PsbtSendResult {
        psbt_base64: unsigned_hex,
        txid: None,
        status: "awaiting_hardware_signature".into(),
    })
}

pub async fn finalize_and_broadcast_light(
    state: &AppState,
    coin: CoinId,
    psbt_base64: &str,
) -> AppResult<String> {
    let prefs = crate::prefs::load().await?;
    if !prefs.wallet_mode.is_light() {
        return Err(AppError::other("not in light wallet mode"));
    }
    let raw = psbt_base64.trim().to_string();
    let verify = crate::wallet::verify::verify_raw_tx_hex(&raw)?;
    if !verify.ok {
        return Err(AppError::other(verify.detail));
    }
    let backend = crate::wallet::backend::resolve_backend(state, coin).await?;
    backend.broadcast_tx(&raw).await
}
