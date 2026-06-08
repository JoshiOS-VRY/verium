//! Enrich recent-block rows from the local node (`getblock` v2). Production builds
//! do not expose `rpc_raw_call`, so the UI uses these commands instead.

use serde_json::{json, Value};

use crate::coin_profile::CoinId;
use crate::error::AppResult;
use crate::explorer_api::ExplorerBlock;
use crate::state::AppState;

fn vout_address(vout: &Value) -> Option<String> {
    let spk = vout.get("scriptPubKey")?;
    if let Some(addr) = spk.get("address").and_then(Value::as_str) {
        return Some(addr.to_string());
    }
    spk.get("addresses")
        .and_then(Value::as_array)
        .and_then(|addrs| addrs.first())
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn parse_f64(v: &Value) -> Option<f64> {
    v.as_f64()
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
}

fn format_output_amount(total: f64) -> String {
    // Trim trailing zeros like the wallet UI (e.g. 1.3135 not 1.31350000).
    let s = format!("{total:.8}");
    let trimmed = s.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

fn sum_outputs_and_miner(
    block: &Value,
    coin: CoinId,
) -> (Option<String>, Option<String>, u64) {
    let Some(txs) = block.get("tx").and_then(Value::as_array) else {
        return (None, None, 0);
    };

    let mut total = 0f64;
    let mut count = 0u64;
    let mut miner = None;

    for tx in txs {
        let vins = tx.get("vin").and_then(Value::as_array);
        let is_coinbase = vins.is_some_and(|vins| {
            vins.iter()
                .any(|vin| vin.get("coinbase").is_some())
        });
        let is_coinstake = coin == CoinId::Vericoin
            && vins.is_some_and(|vins| {
                vins.iter()
                    .any(|vin| vin.get("coinstake").is_some())
            });

        let Some(vouts) = tx.get("vout").and_then(Value::as_array) else {
            continue;
        };

        for vout in vouts {
            if let Some(v) = vout.get("value").and_then(parse_f64) {
                total += v;
                count += 1;
            }
            if (is_coinbase || is_coinstake) && miner.is_none() {
                miner = vout_address(vout);
            }
        }
    }

    let output = (total > 0.0).then(|| format_output_amount(total));
    (output, miner, count)
}

fn parse_block_json(
    coin: CoinId,
    height: u64,
    hash: String,
    block: Value,
) -> ExplorerBlock {
    let (output_total, miner_address, output_count) = sum_outputs_and_miner(&block, coin);
    let n_tx = block
        .get("nTx")
        .and_then(|v| v.as_u64())
        .or_else(|| {
            block
                .get("tx")
                .and_then(Value::as_array)
                .map(|txs| txs.len() as u64)
        });
    let difficulty = block.get("difficulty").map(|v| match v {
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        _ => v.to_string(),
    });

    ExplorerBlock {
        id: height,
        hash,
        height,
        time: block.get("time").and_then(|v| v.as_u64()).unwrap_or(0),
        mint: output_total.clone(),
        difficulty,
        n_tx,
        miner_address,
        size: block.get("size").and_then(|v| v.as_u64()),
        output_total,
        output_count: (output_count > 0).then_some(output_count),
    }
}

pub async fn fetch_local_block_row(
    state: &AppState,
    coin: CoinId,
    height: u64,
) -> AppResult<Option<ExplorerBlock>> {
    if height == 0 {
        return Ok(None);
    }

    let client = state.rpc_client(coin).await?;
    let hash: String = client.call("getblockhash", json!([height])).await?;
    let block: Value = client.call("getblock", json!([&hash, 2])).await?;
    Ok(Some(parse_block_json(coin, height, hash, block)))
}

pub async fn fetch_local_blocks_enriched(
    state: &AppState,
    coin: CoinId,
    heights: Vec<u64>,
) -> AppResult<Vec<ExplorerBlock>> {
    let mut rows = Vec::with_capacity(heights.len());
    for height in heights {
        if let Some(row) = fetch_local_block_row(state, coin, height).await? {
            rows.push(row);
        }
    }
    Ok(rows)
}
