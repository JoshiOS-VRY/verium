//! Background per-coin watcher that long-polls the local node for new chain
//! tips and pushes them to the UI via the `chain-tip-changed` event. This
//! replaces explorer polling for live block updates: the desktop app always
//! runs a node, and `waitfornewblock` wakes the moment any block is connected.

use std::time::Duration;

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};
use tokio::time::sleep;

use crate::coin_profile::CoinId;
use crate::commands::rpc_reachable;
use crate::explorer_api::invalidate_blocks_cache;
use crate::memory_telemetry::{track_background_task_end, track_background_task_start};
use crate::rpc::RpcClient;
use crate::state::AppState;

/// Long-poll window passed to `waitfornewblock`. The node returns the current
/// tip on timeout, so we re-evaluate roughly this often even when idle.
const WAIT_TIMEOUT_MS: u64 = 50_000;

/// HTTP timeout for the watcher client. Must comfortably exceed the long-poll
/// window since `waitfornewblock` holds the request open until it fires.
const WATCHER_RPC_TIMEOUT: Duration = Duration::from_secs(70);

/// Backoff while the node is unreachable or a call fails.
const RETRY_BACKOFF: Duration = Duration::from_secs(5);

/// Minimum gap between `chain-tip-changed` events while the node is flushing blocks.
/// Without this, catch-up sync can emit dozens of tips per second and overwhelm the
/// WebView / React layer (observed as STATUS_STACK_BUFFER_OVERRUN in dev builds).
const MIN_EMIT_INTERVAL: Duration = Duration::from_millis(1500);

/// Idle wait when the coin is disabled in preferences.
const DISABLED_BACKOFF: Duration = Duration::from_secs(30);

/// Spawn one watcher task per coin. Each task runs for the app lifetime,
/// reconnecting on its own when the node restarts.
pub fn spawn_chain_tip_watchers(app: AppHandle, state: AppState) {
    for coin in CoinId::all() {
        let app = app.clone();
        let state = state.clone();
        let coin = *coin;
        tauri::async_runtime::spawn(async move {
            track_background_task_start();
            watch_loop(app, state, coin).await;
            track_background_task_end();
        });
    }
}

async fn watch_loop(app: AppHandle, state: AppState, coin: CoinId) {
    let mut last_hash: Option<String> = None;
    let mut last_emit = tokio::time::Instant::now() - MIN_EMIT_INTERVAL;

    loop {
        let prefs = crate::prefs::load().await.unwrap_or_default();
        if !crate::prefs::coin_enabled(&prefs, coin) {
            sleep(DISABLED_BACKOFF).await;
            continue;
        }
        // Light-mode coins have no managed local node to long-poll; the Electrum
        // backend drives their tip. Skip the watcher so it does not retry RPC
        // against an absent daemon every few seconds.
        if crate::prefs::wallet_mode_for(&prefs, coin).is_light() {
            sleep(DISABLED_BACKOFF).await;
            continue;
        }

        let cfg = match state.config_fresh(coin).await {
            Ok(c) => c,
            Err(_) => {
                sleep(RETRY_BACKOFF).await;
                continue;
            }
        };

        if !rpc_reachable(coin, &cfg).await {
            sleep(RETRY_BACKOFF).await;
            continue;
        }

        let client = match RpcClient::from_config_for_coin_with_timeout(
            coin,
            &cfg,
            WATCHER_RPC_TIMEOUT,
        ) {
            Ok(c) => c,
            Err(_) => {
                sleep(RETRY_BACKOFF).await;
                continue;
            }
        };

        match current_tip(&client).await {
            Some((height, hash)) if last_hash.as_deref() != Some(hash.as_str()) => {
                last_hash = Some(hash.clone());
                if last_emit.elapsed() >= MIN_EMIT_INTERVAL {
                    emit_tip(&app, &client, coin, height, &hash).await;
                    last_emit = tokio::time::Instant::now();
                }
                // Brief pause so catch-up sync cannot tight-loop RPC + IPC.
                sleep(Duration::from_millis(400)).await;
                continue;
            }
            Some(_) => {
                // Tip unchanged: block until the node reports a new one.
                wait_for_new_block(&client).await;
            }
            None => {
                sleep(RETRY_BACKOFF).await;
            }
        }
    }
}

/// Fetch the current tip cheaply via `getblockchaininfo`.
async fn current_tip(client: &RpcClient) -> Option<(u64, String)> {
    let info: Value = client.call("getblockchaininfo", json!([])).await.ok()?;
    let height = info.get("blocks").and_then(Value::as_u64)?;
    let hash = info
        .get("bestblockhash")
        .and_then(Value::as_str)?
        .to_string();
    Some((height, hash))
}

/// Block until the node connects a new tip (or the long-poll times out).
async fn wait_for_new_block(client: &RpcClient) {
    let _: Result<Value, _> = client
        .call("waitfornewblock", json!([WAIT_TIMEOUT_MS]))
        .await;
}

async fn emit_tip(
    app: &AppHandle,
    client: &RpcClient,
    coin: CoinId,
    height: u64,
    hash: &str,
) {
    // Lightweight header lookup only — avoid getblock verbosity 2 (full block + txs)
    // which allocates megabytes per tip and crosses IPC to the WebView.
    let time = block_header_time(client, hash).await.unwrap_or(0);

    invalidate_blocks_cache(coin).await;

    let payload = json!({
        "coin": coin.as_str(),
        "height": height,
        "hash": hash,
        "time": time,
    });
    let _ = app.emit("chain-tip-changed", payload);
}

async fn block_header_time(client: &RpcClient, hash: &str) -> Option<u64> {
    let header: Value = client.call("getblockheader", json!([hash])).await.ok()?;
    header.get("time").and_then(Value::as_u64)
}
