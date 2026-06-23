//! Node/blockchain status command.
//!
//! Tauri-free port of `commands::get_node_status`. Composes `getblockchaininfo`
//! + `getnetworkinfo` into the [`NodeStatus`] DTO the dashboard consumes.

use crate::coin::CoinId;
use crate::context::AppContext;
use crate::error::{HostError, HostResult};
use crate::model::NodeStatus;
use crate::rpc::RpcClient;

/// Fetch combined node status for `coin`.
///
/// This is the function the Phase 0 spike's stub stands in for; the
/// `NodeController` will call it directly once the endpoint is wired.
pub async fn get_node_status(ctx: &AppContext, coin: CoinId) -> HostResult<NodeStatus> {
    let endpoint = match ctx.endpoint(coin) {
        Some(ep) => ep,
        None => {
            // No daemon configured yet (fresh install / pre-setup): show Offline
            // rather than erroring, matching the Tauri dashboard.
            return Ok(NodeStatus {
                connected: false,
                state: "Offline".to_string(),
                ..Default::default()
            });
        }
    };
    let client = RpcClient::new(ctx.http(), &endpoint);

    let chain_info = match client.call("getblockchaininfo", serde_json::json!([])).await {
        Ok(v) => v,
        Err(e) if e.is_warmup() => {
            return Ok(NodeStatus {
                connected: true,
                warming_up: true,
                state: "Warming up".to_string(),
                ..Default::default()
            });
        }
        Err(HostError::Http(_)) => {
            // Daemon not reachable yet.
            return Ok(NodeStatus {
                connected: false,
                state: "Offline".to_string(),
                ..Default::default()
            });
        }
        Err(e) => return Err(e),
    };

    let net_info = client
        .call("getnetworkinfo", serde_json::json!([]))
        .await
        .unwrap_or(serde_json::Value::Null);

    let blocks = chain_info.get("blocks").and_then(|v| v.as_i64()).unwrap_or(0);
    let headers = chain_info
        .get("headers")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let progress = chain_info
        .get("verificationprogress")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let ibd = chain_info
        .get("initialblockdownload")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let median_time = chain_info
        .get("mediantime")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let connections = net_info
        .get("connections")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let state = if ibd || (headers > 0 && blocks < headers) {
        "Syncing"
    } else if progress >= 0.9999 {
        "Synced"
    } else {
        "Connecting"
    }
    .to_string();

    Ok(NodeStatus {
        connected: true,
        warming_up: false,
        chain: chain_info
            .get("chain")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        blocks,
        headers,
        verification_progress: progress,
        initial_block_download: ibd,
        median_time,
        connections,
        state,
    })
}
