//! Fee estimation for light wallet.

use crate::chain::ChainBackend;
use crate::error::AppResult;
use crate::wallet::utxo_selector::{estimate_tx_size, fee_for_rate};

pub async fn estimate_send_fee(
    backend: &dyn ChainBackend,
    input_count: usize,
    output_count: usize,
    target_blocks: u32,
) -> AppResult<i64> {
    let rate = backend.estimate_fee(target_blocks).await?;
    Ok(fee_for_rate(rate.coins_per_kb, input_count, output_count))
}

pub fn estimate_size(input_count: usize, output_count: usize) -> usize {
    estimate_tx_size(input_count, output_count)
}
