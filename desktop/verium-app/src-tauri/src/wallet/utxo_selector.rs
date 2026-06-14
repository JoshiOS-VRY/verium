//! Simple UTXO selection for light wallet sends (SDK wrapper).

use crate::chain::types::Utxo;
use crate::error::AppResult;
use crate::sdk_bridge::map_wallet_err;

pub use vericonomy_wallet_engine::utxo_selector::{
    coins_per_kb_to_sats_per_k, estimate_tx_size, verium_fee_for_size, DEFAULT_TX_FEE_COINS_PER_KB,
    DUST_CHANGE_SATS, VIP1_MIN_TX_FEE_PER_K,
};

pub fn select_utxos(utxos: &[Utxo], target_sats: i64, fee_sats: i64) -> AppResult<Vec<Utxo>> {
    map_wallet_err(vericonomy_wallet_engine::utxo_selector::select_utxos(
        utxos, target_sats, fee_sats,
    ))
}

pub fn fee_for_rate(rate_coins_per_kb: f64, input_count: usize, output_count: usize) -> i64 {
    vericonomy_wallet_engine::utxo_selector::fee_for_rate(
        rate_coins_per_kb,
        input_count,
        output_count,
    )
}

pub fn plan_send_utxos(
    utxos: &[Utxo],
    amount_sats: i64,
    rate_coins_per_kb: f64,
    recipient_outputs: usize,
) -> AppResult<(Vec<Utxo>, i64)> {
    map_wallet_err(vericonomy_wallet_engine::utxo_selector::plan_send_utxos(
        utxos,
        amount_sats,
        rate_coins_per_kb,
        recipient_outputs,
    ))
}

pub fn replan_fee_for_selected(
    selected: &[Utxo],
    amount_sats: i64,
    rate_coins_per_kb: f64,
    recipient_outputs: usize,
) -> AppResult<i64> {
    map_wallet_err(
        vericonomy_wallet_engine::utxo_selector::replan_fee_for_selected(
            selected,
            amount_sats,
            rate_coins_per_kb,
            recipient_outputs,
        ),
    )
}
