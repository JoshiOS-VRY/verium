//! Local transaction verification for light wallet (decode + output check).

use crate::coin_profile::CoinId;
use crate::error::{AppError, AppResult};
use crate::wallet::hd;
use crate::wallet::vericonomy_tx::decode_verium_tx;

#[derive(Debug, Clone)]
pub struct TxVerifyResult {
    pub ok: bool,
    pub detail: String,
}

pub fn verify_raw_tx_hex(raw_hex: &str) -> AppResult<TxVerifyResult> {
    let bytes = hex::decode(raw_hex.trim())
        .map_err(|e| AppError::other(format!("tx hex decode: {e}")))?;
    let tx = decode_verium_tx(&bytes).map_err(|e| AppError::other(format!("tx decode: {e}")))?;
    if tx.inputs.is_empty() || tx.outputs.is_empty() {
        return Ok(TxVerifyResult {
            ok: false,
            detail: "transaction has no inputs or outputs".into(),
        });
    }
    Ok(TxVerifyResult {
        ok: true,
        detail: format!("{} inputs, {} outputs", tx.inputs.len(), tx.outputs.len()),
    })
}

pub fn script_pays_to(script_hex: &str, expected_script_hex: &str) -> bool {
    script_hex.trim().eq_ignore_ascii_case(expected_script_hex.trim())
}

/// Verify signed transaction outputs match user-approved destinations and amounts.
pub fn verify_send_outputs(
    coin: CoinId,
    raw_hex: &str,
    expected: &[(String, i64)],
) -> AppResult<()> {
    let bytes = hex::decode(raw_hex.trim())
        .map_err(|e| AppError::other(format!("tx hex decode: {e}")))?;
    let tx = decode_verium_tx(&bytes).map_err(|e| AppError::other(format!("tx decode: {e}")))?;
    if tx.inputs.is_empty() || tx.outputs.is_empty() {
        return Err(AppError::other(
            "transaction has no inputs or outputs",
        ));
    }
    for (addr, amount_sats) in expected {
        if *amount_sats <= 0 {
            return Err(AppError::other(format!("invalid amount for {addr}")));
        }
        let script = hd::address_to_script_pubkey(coin, addr)?;
        let amount_u64 = u64::try_from(*amount_sats)
            .map_err(|_| AppError::other(format!("amount overflow for {addr}")))?;
        let matched = tx.outputs.iter().any(|out| {
            out.script_pubkey.as_bytes() == script.as_slice()
                && out.value.to_sat() == amount_u64
        });
        if !matched {
            return Err(AppError::other(format!(
                "signed transaction does not pay {amount_sats} sats to {addr}"
            )));
        }
    }
    Ok(())
}
