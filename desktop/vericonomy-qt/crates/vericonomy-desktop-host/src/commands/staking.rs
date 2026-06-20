//! Staking RPC: getstakinginfo, stakingstart, stakingstop.

use crate::coin::CoinId;
use crate::context::AppContext;
use crate::error::{HostError, HostResult};
use crate::model::{EarnState, StakingInfo};
use crate::rpc::RpcClient;

pub async fn get_staking_info(ctx: &AppContext, coin: CoinId) -> HostResult<StakingInfo> {
    if coin != CoinId::Vericoin {
        return Ok(StakingInfo::default());
    }
    let endpoint = match ctx.endpoint(coin) {
        Some(ep) => ep,
        None => return Ok(StakingInfo::default()),
    };
    let client = RpcClient::new(ctx.http(), &endpoint);
    let staking = client
        .call("getstakinginfo", serde_json::json!([]))
        .await
        .unwrap_or(serde_json::Value::Null);
    let earn = ctx.earn(coin);
    Ok(StakingInfo {
        enabled: earn.active
            || staking
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        stake_weight: staking
            .get("stakeweight")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        netstakeweight: staking
            .get("netstakeweight")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        expected_time: staking
            .get("expectedtime")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        searches_per_second: staking
            .get("searchespersecond")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
    })
}

pub async fn staking_start(ctx: &AppContext, coin: CoinId) -> HostResult<EarnState> {
    if coin != CoinId::Vericoin {
        return Err(HostError::other("staking is Vericoin only"));
    }
    let endpoint = ctx
        .endpoint(coin)
        .ok_or_else(|| HostError::other("no RPC endpoint"))?;
    let client = RpcClient::new(ctx.http(), &endpoint);
    let _: serde_json::Value = client.call("stakingstart", serde_json::json!([])).await?;
    let state = EarnState {
        active: true,
        threads: 0,
    };
    ctx.set_earn(coin, state.clone());
    Ok(state)
}

pub async fn staking_stop(ctx: &AppContext, coin: CoinId) -> HostResult<EarnState> {
    if coin != CoinId::Vericoin {
        return Err(HostError::other("staking is Vericoin only"));
    }
    if let Some(ep) = ctx.endpoint(coin) {
        let client = RpcClient::new(ctx.http(), &ep);
        let _ = client.call("stakingstop", serde_json::json!([])).await;
    }
    let state = EarnState::default();
    ctx.set_earn(coin, state.clone());
    Ok(state)
}
