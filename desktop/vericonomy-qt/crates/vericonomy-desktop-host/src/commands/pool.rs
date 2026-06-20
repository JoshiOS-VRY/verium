//! Pool miner API (P3) — cpuminer sidecar.

use crate::coin::CoinId;
use crate::context::AppContext;
use crate::error::{HostError, HostResult};

#[derive(Debug, Clone, serde::Serialize)]
pub struct PoolMinerStatus {
    pub running: bool,
    pub hashrate: f64,
}

pub async fn pool_miner_status(_ctx: &AppContext, coin: CoinId) -> HostResult<PoolMinerStatus> {
    if coin != CoinId::Verium {
        return Ok(PoolMinerStatus {
            running: false,
            hashrate: 0.0,
        });
    }
    Err(HostError::other("Pool miner sidecar not yet ported (P1/P3)"))
}
