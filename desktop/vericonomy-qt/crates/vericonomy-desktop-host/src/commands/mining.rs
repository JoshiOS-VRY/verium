//! Mining RPC: getmininginfo, minerstart, minerstop.

use crate::coin::CoinId;
use crate::context::AppContext;
use crate::error::HostResult;
use crate::model::{EarnState, MiningInfo};
use crate::rpc::RpcClient;

pub async fn get_mining_info(ctx: &AppContext, coin: CoinId) -> HostResult<MiningInfo> {
    let endpoint = match ctx.endpoint(coin) {
        Some(ep) => ep,
        None => return Ok(MiningInfo::default()),
    };
    let client = RpcClient::new(ctx.http(), &endpoint);
    let v = client.call("getmininginfo", serde_json::json!([])).await?;
    Ok(MiningInfo {
        blocks: v.get("blocks").and_then(|x| x.as_i64()).unwrap_or(0),
        currentblocksize: v.get("currentblocksize").and_then(|x| x.as_i64()).unwrap_or(0),
        currentblocktx: v.get("currentblocktx").and_then(|x| x.as_i64()).unwrap_or(0),
        difficulty: v.get("difficulty").and_then(|x| x.as_f64()).unwrap_or(0.0),
        networkhashps: v.get("networkhashps").and_then(|x| x.as_f64()).unwrap_or(0.0),
        hashrate: v.get("hashrate").and_then(|x| x.as_f64()).unwrap_or(0.0),
        pooledtx: v.get("pooledtx").and_then(|x| x.as_i64()).unwrap_or(0),
        blocksperhour: v.get("blocksperhour").and_then(|x| x.as_f64()).unwrap_or(0.0),
        blocktime: v.get("blocktime").and_then(|x| x.as_f64()).unwrap_or(0.0),
        blockreward: v.get("blockreward").and_then(|x| x.as_f64()).unwrap_or(0.0),
        chain: v
            .get("chain")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
        warnings: v
            .get("warnings")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
    })
}

pub async fn miner_start(
    ctx: &AppContext,
    coin: CoinId,
    threads: u32,
    reward_address: Option<&str>,
) -> HostResult<EarnState> {
    let endpoint = ctx.endpoint(coin).ok_or_else(|| {
        crate::error::HostError::other("mining requires a connected Verium node")
    })?;
    let client = RpcClient::new(ctx.http(), &endpoint);
    let mut params = vec![serde_json::json!(threads)];
    if let Some(addr) = reward_address.filter(|s| !s.is_empty()) {
        params.push(serde_json::json!(addr));
    }
    let _: serde_json::Value = client.call("minerstart", serde_json::json!(params)).await?;
    let state = EarnState {
        active: true,
        threads,
    };
    ctx.set_earn(coin, state.clone());
    Ok(state)
}

pub async fn miner_stop(ctx: &AppContext, coin: CoinId) -> HostResult<EarnState> {
    if let Some(ep) = ctx.endpoint(coin) {
        let client = RpcClient::new(ctx.http(), &ep);
        let _ = client.call("minerstop", serde_json::json!([])).await;
    }
    let state = EarnState::default();
    ctx.set_earn(coin, state.clone());
    Ok(state)
}

pub fn get_miner_state(ctx: &AppContext, coin: CoinId) -> EarnState {
    ctx.earn(coin)
}
