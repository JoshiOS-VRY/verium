//! Explorer HTTP API (stats + recent blocks).

use crate::coin::CoinId;
use crate::context::AppContext;
use crate::error::{HostError, HostResult};
use crate::model::{ExplorerBlock, ExplorerStats};

pub async fn fetch_explorer_stats(ctx: &AppContext, coin: CoinId) -> HostResult<ExplorerStats> {
    let url = format!("{}/stats", coin.explorer_api_base());
    let resp = ctx.http().get(&url).send().await?;
    let v: serde_json::Value = resp.json().await?;
    Ok(ExplorerStats {
        network_hash: v.get("network_hash").and_then(|x| x.as_f64()),
        supply: v.get("supply").and_then(|x| x.as_f64()),
        height: v.get("height").and_then(|x| x.as_u64()),
        difficulty: v.get("difficulty").and_then(|x| x.as_f64()),
        price_usd: v.get("price_usd").and_then(|x| x.as_f64()),
        volume_24h_usd: v.get("volume_24h_usd").and_then(|x| x.as_f64()),
        block_reward: v.get("block_reward").and_then(|x| x.as_f64()),
        stake_interest: v.get("stake_interest").and_then(|x| x.as_f64()),
        pooled_tx: v.get("pooled_tx").and_then(|x| x.as_i64()),
        blocks_per_hour: v.get("blocks_per_hour").and_then(|x| x.as_f64()),
        block_time_min: v.get("block_time_min").and_then(|x| x.as_f64()),
        source: "explorer".into(),
    })
}

pub async fn fetch_explorer_blocks(
    ctx: &AppContext,
    coin: CoinId,
    limit: u32,
) -> HostResult<Vec<ExplorerBlock>> {
    let url = format!(
        "{}/blocks/latest?limit={}",
        coin.explorer_chain_api_base(),
        limit.clamp(1, 25)
    );
    let resp = ctx.http().get(&url).send().await?;
    let v: serde_json::Value = resp.json().await?;
    let arr = v
        .as_array()
        .ok_or_else(|| HostError::other("blocks response not array"))?;
    Ok(arr
        .iter()
        .filter_map(|item| {
            Some(ExplorerBlock {
                height: item.get("height")?.as_u64()?,
                hash: item.get("hash")?.as_str()?.to_string(),
                time: item.get("time").and_then(|t| t.as_u64()).unwrap_or(0),
                n_tx: item.get("txCount").and_then(|t| t.as_u64()),
                miner_address: item
                    .get("extractedByAddress")
                    .or_else(|| item.get("miner_address"))
                    .and_then(|t| t.as_str())
                    .map(str::to_string),
                size: item.get("size").and_then(|t| t.as_u64()),
                difficulty: item
                    .get("difficulty")
                    .and_then(|t| t.as_str())
                    .map(str::to_string),
                output_total: item
                    .get("output_total")
                    .and_then(|t| t.as_str())
                    .map(str::to_string),
                mint: item.get("mint").and_then(|t| t.as_str()).map(str::to_string),
            })
        })
        .collect())
}
