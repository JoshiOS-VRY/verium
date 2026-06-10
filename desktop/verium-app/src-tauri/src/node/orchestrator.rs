use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;
use tauri::{AppHandle, Emitter};
use tokio::time::sleep;

use crate::coin_profile::CoinId;
use crate::commands::{
    daemon_boot_in_progress, ensure_daemon_running, reindex_running_live, startup_prepare_chain_data,
    stop_inner,
};
use crate::commands::{heal_invalid_blocks_silently, rpc_reachable};
use crate::node::constants::{INVALID_BLOCK_HEAL_TICK, SUPERVISOR_TICK};
use crate::node::state::NodeSnapshot;
use crate::network_mode_commands;
use crate::prefs;
use crate::rpc::RpcClient;
use crate::state::AppState;

static LAST_EMITTED: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

/// How long after the last demand signal the managed daemon is kept running.
/// Demand is bumped by UI status polls and wallet activity; solo mining and
/// staking count as continuous demand (checked separately). Once this elapses
/// with no demand and the chain synced, the idle watchdog stops the daemon.
const DAEMON_IDLE_TIMEOUT: Duration = Duration::from_secs(15 * 60);

/// How often the idle watchdog evaluates whether to stop idle daemons.
const AUTO_STOP_CHECK_TICK: Duration = Duration::from_secs(60);

/// Grace period after launch before the idle watchdog may stop anything.
const AUTO_STOP_STARTUP_GRACE: Duration = Duration::from_secs(120);

/// Per-coin timestamp of the last "the daemon is needed" signal.
static DAEMON_DEMAND_AT: Lazy<Mutex<HashMap<CoinId, Instant>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Record that something needs the managed daemon for `coin` right now. Called
/// from UI status polls and wallet activity so the daemon is started on demand
/// and kept alive while in use.
pub fn note_daemon_demand(coin: CoinId) {
    if let Ok(mut map) = DAEMON_DEMAND_AT.lock() {
        map.insert(coin, Instant::now());
    }
}

/// Drop demand for a coin (e.g. when the user switches to the other chain).
pub fn clear_daemon_demand(coin: CoinId) {
    if let Ok(mut map) = DAEMON_DEMAND_AT.lock() {
        map.remove(&coin);
    }
}

/// True when demand was registered within `DAEMON_IDLE_TIMEOUT`.
fn daemon_demand_recent(coin: CoinId) -> bool {
    DAEMON_DEMAND_AT
        .lock()
        .ok()
        .and_then(|map| map.get(&coin).map(|at| at.elapsed() < DAEMON_IDLE_TIMEOUT))
        .unwrap_or(false)
}

fn snapshot_key(coin: CoinId, snap: &NodeSnapshot) -> String {
    format!(
        "{}:{}:{}",
        coin.as_str(),
        format!("{:?}", snap.state),
        snap.message
    )
}

pub fn maybe_emit_state(app: &AppHandle, coin: CoinId, snap: &NodeSnapshot) {
    let key = snapshot_key(coin, snap);
    let mut guard = LAST_EMITTED.lock().unwrap();
    let map = guard.get_or_insert_with(HashMap::new);
    if map.get(coin.as_str()) == Some(&key) {
        return;
    }
    map.insert(coin.as_str().to_string(), key);
    let payload = serde_json::json!({
        "coin": coin.as_str(),
        "snapshot": snap,
    });
    let _ = app.emit("node-state-changed", payload);
}

/// Single entry point for app startup: prepare chain, ensure daemons, start supervisor.
pub async fn startup(app: AppHandle, state: &AppState) {
    if let Err(e) = network_mode_commands::ensure_mainnet_when_binarytest_disabled(state).await
    {
        tracing::warn!("startup: binarytest→mainnet migration failed: {e}");
    }

    let prefs = prefs::load().await.unwrap_or_default();

    // Per-coin wallet mode: a coin in light mode never gets a managed daemon;
    // a coin in full-node mode follows the daemon prepare/ensure/wait pipeline.
    // This lets e.g. Verium run a full node while Vericoin runs light.
    let light_coins: Vec<CoinId> = CoinId::all()
        .iter()
        .copied()
        .filter(|c| prefs::wallet_mode_for(&prefs, *c).is_light())
        .collect();
    let full_node_coins: Vec<CoinId> = CoinId::all()
        .iter()
        .copied()
        .filter(|c| {
            prefs::coin_enabled(&prefs, *c) && !prefs::wallet_mode_for(&prefs, *c).is_light()
        })
        .collect();

    for coin in &light_coins {
        if crate::wallet::keystore::is_unlocked(*coin).unwrap_or(false)
            && !crate::wallet::keystore::signing_session_active(*coin)
        {
            tracing::info!(
                "startup: clearing stale light wallet unlock for {} (session not active)",
                coin.as_str()
            );
            let _ = crate::wallet::keystore::lock_wallet(*coin);
        }
    }
    if !light_coins.is_empty() {
        let sync_state = state.clone();
        let sync_coins = light_coins.clone();
        tauri::async_runtime::spawn(async move {
            for coin in sync_coins {
                if crate::wallet::keystore::light_wallet_on_disk(coin) {
                    let _ = crate::wallet::sync::sync_light_wallet(&sync_state, coin).await;
                }
            }
        });
    }

    if full_node_coins.is_empty() {
        tracing::info!("startup: no full-node coins — skipping daemon orchestration");
        return;
    }

    // Prepare chain data dirs/config up front (cheap, no process spawn) so a
    // later on-demand start is fast. We intentionally do NOT eagerly start the
    // daemon here: it is started lazily by the supervisor once demand appears
    // (the UI registers demand via get_node_status as soon as a full-node view
    // mounts), and stopped again by the idle watchdog when nothing needs it.
    for coin in &full_node_coins {
        if let Err(e) = startup_prepare_chain_data(state, *coin).await {
            tracing::warn!(
                "startup ({}): chain data prepare failed: {e}",
                coin.as_str()
            );
        }
    }
    // Keep every enabled full-node daemon warm so coin switches do not stop peers.
    for coin in &full_node_coins {
        note_daemon_demand(*coin);
    }

    let supervisor_state = state.clone();
    let supervisor_app = app.clone();
    tauri::async_runtime::spawn(async move {
        supervisor_loop(&supervisor_app, &supervisor_state).await;
    });

    let heal_state = state.clone();
    tauri::async_runtime::spawn(async move {
        invalid_block_heal_loop(&heal_state).await;
    });

    let autostop_state = state.clone();
    tauri::async_runtime::spawn(async move {
        daemon_idle_autostop_loop(&autostop_state).await;
    });

    crate::chain_tip_watcher::spawn_chain_tip_watchers(app, state.clone());
}

/// Stop managed daemons that have had no demand for `DAEMON_IDLE_TIMEOUT` and
/// are safe to stop (chain synced, not mining/staking, not reindexing/booting).
/// This reclaims the daemon's RAM/CPU when the wallet is idle or backgrounded;
/// the supervisor restarts it on the next demand.
async fn daemon_idle_autostop_loop(state: &AppState) {
    sleep(AUTO_STOP_STARTUP_GRACE).await;
    loop {
        sleep(AUTO_STOP_CHECK_TICK).await;
        let prefs = prefs::load().await.unwrap_or_default();
        for coin in CoinId::all() {
            let coin = *coin;
            if !prefs::coin_enabled(&prefs, coin)
                || prefs::wallet_mode_for(&prefs, coin).is_light()
            {
                continue;
            }
            if daemon_demand_recent(coin) || state.earn_active(coin).await {
                continue;
            }
            let cfg = match state.config_fresh(coin).await {
                Ok(c) => c,
                Err(_) => continue,
            };
            // Don't interfere with in-flight node work.
            if state.bootstrap_session_active()
                || reindex_running_live(state, coin, &cfg).await
                || daemon_boot_in_progress(state, coin, &cfg).await
            {
                continue;
            }
            if !rpc_reachable(coin, &cfg).await {
                continue; // already stopped / not running
            }
            // Never stop mid initial-block-download — that would strand sync.
            if !chain_fully_synced(coin, &cfg).await {
                continue;
            }
            tracing::info!(
                "idle-autostop ({}): no demand for {}s — stopping managed daemon",
                coin.as_str(),
                DAEMON_IDLE_TIMEOUT.as_secs()
            );
            if let Err(e) = stop_inner(state, coin).await {
                tracing::warn!("idle-autostop ({}): stop failed: {e}", coin.as_str());
            } else {
                state.set_daemon_phase(coin, "idle_stopped");
            }
        }
    }
}

/// Cheap synced check used before idle-stop: connected, not in IBD.
async fn chain_fully_synced(coin: CoinId, cfg: &crate::config::DaemonConfig) -> bool {
    let Ok(client) = RpcClient::status_client_for_coin(coin, cfg) else {
        return false;
    };
    let Ok(info) = client
        .call::<serde_json::Value>("getblockchaininfo", serde_json::json!([]))
        .await
    else {
        return false;
    };
    let ibd = info
        .get("initialblockdownload")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let progress = info
        .get("verificationprogress")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    !ibd && progress >= 0.9999
}

/// When the user switches chains, refresh daemon demand for every enabled
/// full-node coin so neither peer is stopped by a switch or the idle watchdog.
pub async fn on_active_coin_changed(_state: &AppState, new_active: CoinId) {
    let prefs = prefs::load().await.unwrap_or_default();
    note_daemon_demand(new_active);
    for coin in CoinId::all() {
        if !prefs::coin_enabled(&prefs, *coin)
            || prefs::wallet_mode_for(&prefs, *coin).is_light()
        {
            continue;
        }
        note_daemon_demand(*coin);
    }
    tracing::debug!(
        "active coin → {}: keeping all enabled full-node daemons running",
        new_active.as_str()
    );
}

/// Proactively clear invalid block flags before status polling can surface a stall banner.
async fn invalid_block_heal_loop(state: &AppState) {
    sleep(Duration::from_secs(20)).await;
    loop {
        let prefs = prefs::load().await.unwrap_or_default();
        for coin in CoinId::all() {
            if !prefs::coin_enabled(&prefs, *coin) {
                continue;
            }
            // Light-mode coins run no managed daemon, so there is nothing to heal
            // and no RPC endpoint to probe — skip them entirely.
            if prefs::wallet_mode_for(&prefs, *coin).is_light() {
                continue;
            }
            let cfg = match state.config_fresh(*coin).await {
                Ok(c) => c,
                Err(_) => continue,
            };
            if !rpc_reachable(*coin, &cfg).await {
                continue;
            }
            let _ = heal_invalid_blocks_silently(state, *coin, &cfg).await;
        }
        sleep(INVALID_BLOCK_HEAL_TICK).await;
    }
}

async fn supervisor_loop(app: &AppHandle, state: &AppState) {
    sleep(Duration::from_secs(5)).await;
    loop {
        let prefs = prefs::load().await.unwrap_or_default();
        for coin in CoinId::all() {
            if prefs::coin_enabled(&prefs, *coin)
                && !prefs::wallet_mode_for(&prefs, *coin).is_light()
            {
                note_daemon_demand(*coin);
            }
        }
        for coin in CoinId::all() {
            supervise_coin(app, state, *coin).await;
        }
        sleep(SUPERVISOR_TICK).await;
    }
}

async fn supervise_coin(_app: &AppHandle, state: &AppState, coin: CoinId) {
    use crate::commands::{
        bootstrap_suppresses_auto_start, daemon_boot_in_progress, reindex_running_live,
        restart_daemon_full_cycle, rpc_auth_failed, rpc_reachable,
    };
    use crate::config::{ensure_daemon_conf_complete, sync_cfg_rpc_credentials_from_conf};

    let prefs = prefs::load().await.unwrap_or_default();
    if !prefs::coin_enabled(&prefs, coin) {
        return;
    }
    // Light-mode coins have no managed daemon; never try to start one for them.
    if prefs::wallet_mode_for(&prefs, coin).is_light() {
        return;
    }
    let cfg = match state.config_fresh(coin).await {
        Ok(c) => c,
        Err(e) => {
            tracing::debug!("supervisor ({}): config load failed: {e}", coin.as_str());
            return;
        }
    };

    if state.bootstrap_session_active()
        || bootstrap_suppresses_auto_start(state, coin, &cfg).await
    {
        return;
    }

    if rpc_reachable(coin, &cfg).await {
        state.clear_auth_restart_attempts(coin);
        state.set_daemon_phase(coin, "connected");
        return;
    }

    if reindex_running_live(state, coin, &cfg).await {
        state.set_daemon_phase(coin, "reindexing");
        return;
    }

    if daemon_boot_in_progress(state, coin, &cfg).await {
        state.set_daemon_phase(coin, "starting");
        return;
    }

    // Lazy start: only (re)start the managed daemon when something actually
    // needs it — the UI registered demand recently, or solo mining/staking is
    // active. Otherwise leave it stopped so it isn't started at launch or
    // restarted right after the idle watchdog stops it.
    if !daemon_demand_recent(coin) && !state.earn_active(coin).await {
        state.set_daemon_phase(coin, "idle");
        return;
    }

    if !crate::daemon::pids_listening_on_port(cfg.rpc_port).is_empty()
        && rpc_auth_failed(coin, &cfg).await
    {
        if state.pending_reindex_active(coin) || state.bootstrap_loading_active(coin) {
            state.set_daemon_phase(coin, "reindexing");
            return;
        }
        if state.auth_restart_exhausted(coin) {
            tracing::warn!(
                "supervisor ({}): RPC auth rejected — auto-restart budget exhausted; fix credentials in Settings",
                coin.as_str()
            );
            state.set_daemon_phase(coin, "auth_mismatch");
            return;
        }
        tracing::warn!(
            "supervisor ({}): RPC port open but credentials rejected — syncing conf and restarting node",
            coin.as_str()
        );
        state.increment_auth_restart(coin);
        if let Ok(mut fresh) = state.config_fresh(coin).await {
            let _ = sync_cfg_rpc_credentials_from_conf(coin, &mut fresh);
            let _ = ensure_daemon_conf_complete(coin, &mut fresh);
            let _ = state.replace_config(coin, fresh).await;
        }
        if restart_daemon_full_cycle(state, coin).await.is_ok() {
            state.set_daemon_phase(coin, "starting");
        }
        return;
    }

    ensure_daemon_running(state, coin, &cfg).await;
}
