//! Simple UTXO selection for light wallet sends.

use crate::chain::types::Utxo;
use crate::error::{AppError, AppResult};

/// Verium VIP1+ network minimum fee rate: 100000 satoshis per 1000 bytes (0.001 VRM/kB).
/// See `VIP1_MIN_TX_FEE` in `verium/src/validation.h`.
pub const VIP1_MIN_TX_FEE_PER_K: i64 = 100_000;

/// Default wallet fee rate matching the legacy Qt UI and relay policy.
pub const DEFAULT_TX_FEE_COINS_PER_KB: f64 = 0.001;

pub fn select_utxos(utxos: &[Utxo], target_sats: i64, fee_sats: i64) -> AppResult<Vec<Utxo>> {
    let needed = target_sats + fee_sats;
    let mut sorted: Vec<&Utxo> = utxos.iter().collect();
    sorted.sort_by(|a, b| b.value_sats.cmp(&a.value_sats));

    let mut selected = Vec::new();
    let mut sum = 0i64;
    for u in sorted {
        selected.push(u.clone());
        sum += u.value_sats;
        if sum >= needed {
            return Ok(selected);
        }
    }
    Err(AppError::other("insufficient funds"))
}

/// Verium wire-format P2PKH size (includes the extra `nTime` field after version).
pub fn estimate_tx_size(input_count: usize, output_count: usize) -> usize {
    14 + input_count * 148 + output_count * 34
}

/// Verium `CFeeRate::GetFee(nBytes, addStartFee)` from `policy/feerate.cpp`.
pub fn verium_fee_for_size(satoshis_per_k: i64, n_bytes: usize, add_start_fee: bool) -> i64 {
    let n_size = n_bytes as i64;
    let mut n_fee = satoshis_per_k * (n_size / 1000);
    if add_start_fee {
        n_fee += satoshis_per_k;
    }
    if n_fee == 0 && n_size != 0 && satoshis_per_k > 0 {
        n_fee = 1;
    }
    n_fee
}

pub fn fee_for_rate(rate_coins_per_kb: f64, input_count: usize, output_count: usize) -> i64 {
    let bytes = estimate_tx_size(input_count, output_count);
    let user_rate_sats_per_k = coins_per_kb_to_sats_per_k(rate_coins_per_kb);
    let user_fee = verium_fee_for_size(user_rate_sats_per_k, bytes, true);
    let network_min = verium_fee_for_size(VIP1_MIN_TX_FEE_PER_K, bytes, true);
    user_fee.max(network_min)
}

pub fn coins_per_kb_to_sats_per_k(rate_coins_per_kb: f64) -> i64 {
    (rate_coins_per_kb * 100_000_000.0).round() as i64
}

/// Select inputs and compute fee using the final input count (Verium start-fee model).
pub fn plan_send_utxos(
    utxos: &[Utxo],
    amount_sats: i64,
    rate_coins_per_kb: f64,
    output_count: usize,
) -> AppResult<(Vec<Utxo>, i64)> {
    let mut fee_sats = fee_for_rate(rate_coins_per_kb, 1, output_count);
    let mut selected = select_utxos(utxos, amount_sats, fee_sats)?;
    fee_sats = fee_for_rate(rate_coins_per_kb, selected.len(), output_count);
    let needed = amount_sats + fee_sats;
    let selected_sum: i64 = selected.iter().map(|u| u.value_sats).sum();
    if selected_sum < needed {
        selected = select_utxos(utxos, amount_sats, fee_sats)?;
    }
    Ok((selected, fee_sats))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verium_start_fee_is_at_least_one_per_k_rate() {
        let fee = verium_fee_for_size(VIP1_MIN_TX_FEE_PER_K, 230, true);
        assert_eq!(fee, VIP1_MIN_TX_FEE_PER_K);
    }

    #[test]
    fn low_user_rate_still_meets_network_minimum() {
        // 0.0001 VRM/kB would be 2260 sats with the old Bitcoin-only formula.
        let fee = fee_for_rate(0.0001, 1, 2);
        assert_eq!(fee, VIP1_MIN_TX_FEE_PER_K);
    }

    #[test]
    fn default_rate_matches_network_minimum_for_typical_send() {
        let fee = fee_for_rate(DEFAULT_TX_FEE_COINS_PER_KB, 1, 2);
        assert_eq!(fee, VIP1_MIN_TX_FEE_PER_K);
    }
}
