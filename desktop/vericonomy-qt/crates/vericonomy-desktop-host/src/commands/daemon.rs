//! Daemon lifecycle (P1) — start/stop/restart; ports Tauri `daemon.rs` + orchestrator.

use crate::coin::CoinId;
use crate::context::AppContext;
use crate::daemon_config::{self, DaemonConfig};
use crate::error::{HostError, HostResult};

#[derive(Debug, Clone, serde::Serialize)]
pub struct DaemonStatus {
    pub running: bool,
    pub message: String,
}

pub fn get_daemon_config(coin: CoinId) -> HostResult<DaemonConfig> {
    daemon_config::load_daemon_config(coin)
}

pub async fn start_daemon(_ctx: &AppContext, coin: CoinId) -> HostResult<DaemonStatus> {
    let _cfg = daemon_config::load_daemon_config(coin)?;
    // TODO: port DaemonManager::start + ProcessRegistry from verium-app
    Err(HostError::other(
        "Daemon spawn not yet ported — start veriumd manually or use Tauri until P1 completes",
    ))
}

pub async fn stop_daemon(_ctx: &AppContext, coin: CoinId) -> HostResult<()> {
    let _ = coin;
    Err(HostError::other("Daemon stop not yet ported"))
}

pub async fn restart_daemon(ctx: &AppContext, coin: CoinId) -> HostResult<DaemonStatus> {
    let _ = stop_daemon(ctx, coin).await;
    start_daemon(ctx, coin).await
}
