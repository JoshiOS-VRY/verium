//! Daemon lifecycle — start/stop/restart via [`DaemonManager`].

use crate::coin::CoinId;
use crate::context::AppContext;
use crate::daemon_binary::{detect_binary, DaemonBinaryStatus};
use crate::daemon_config::{self, DaemonConfig};
use crate::daemon_manager::{
    manager, native_daemon_image_running, restart_managed_daemon, rpc_reachable,
    start_managed_daemon, stop_managed_daemon,
};
use crate::error::HostResult;

#[derive(Debug, Clone, serde::Serialize)]
pub struct DaemonStatus {
    pub running: bool,
    pub managed: bool,
    pub message: String,
}

pub fn get_daemon_config(coin: CoinId) -> HostResult<DaemonConfig> {
    daemon_config::load_daemon_config(coin)
}

pub fn detect_daemon_binary(coin: CoinId) -> DaemonBinaryStatus {
    detect_binary(coin)
}

async fn status_for(ctx: &AppContext, coin: CoinId) -> HostResult<DaemonStatus> {
    let mut cfg = daemon_config::load_daemon_config(coin)?;
    daemon_config::sync_rpc_from_conf(coin, &mut cfg)?;
    let reachable = rpc_reachable(ctx, coin, &cfg).await;
    let managed = manager(coin).await.is_managed().await;
    let running = reachable || native_daemon_image_running(coin);
    let message = if reachable {
        "RPC connected".into()
    } else if running {
        "Process running (RPC warming up)".into()
    } else {
        detect_binary(coin)
            .missing_hint
            .unwrap_or_else(|| "Daemon stopped".into())
    };
    Ok(DaemonStatus {
        running,
        managed,
        message,
    })
}

pub async fn start_daemon(ctx: &AppContext, coin: CoinId) -> HostResult<DaemonStatus> {
    start_managed_daemon(ctx, coin).await?;
    status_for(ctx, coin).await
}

pub async fn stop_daemon(ctx: &AppContext, coin: CoinId) -> HostResult<()> {
    stop_managed_daemon(ctx, coin).await
}

pub async fn restart_daemon(ctx: &AppContext, coin: CoinId) -> HostResult<DaemonStatus> {
    restart_managed_daemon(ctx, coin).await?;
    status_for(ctx, coin).await
}

pub async fn daemon_status(ctx: &AppContext, coin: CoinId) -> HostResult<DaemonStatus> {
    status_for(ctx, coin).await
}
