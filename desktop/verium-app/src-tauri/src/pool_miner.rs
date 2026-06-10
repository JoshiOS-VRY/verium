//! Pool CPU miner via veriumd in-process Stratum (`poolminerstart` / `getpoolminerinfo`).

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sysinfo::System;
use tauri::State;

use crate::coin_profile::{assert_verium, CoinId};
use crate::cpuminer_topo::{
    cpuminer_recommended_threads, cpuminer_scratchpad_mib, probe_topo,
};
use crate::daemon::{binary_supports_native_pool_mining, resolve_daemon_binary};
use crate::error::{AppError, AppResult};
use crate::mining_supervisor;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolMinerStartConfig {
    pub stratum_url: String,
    pub username: String,
    pub password: Option<String>,
    pub threads: u32,
    /// Optional comma-separated failover pool URL(s) for the sidecar backend.
    #[serde(default)]
    pub backup_url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolMinerStatus {
    pub running: bool,
    /// Local hashrate in H/m (matches solo mining display).
    pub hashrate_hm: f64,
    pub worker: String,
    pub last_log_line: String,
    /// `sidecar` — dedicated cpuminer process; `native` — hashing inside veriumd.
    pub backend: String,
    pub active_threads: u32,
    /// Accepted shares this session (sidecar backend only; 0 for native).
    pub accepted_shares: u64,
    /// Rejected shares this session (sidecar backend only; 0 for native).
    pub rejected_shares: u64,
    /// True when the miner reports a live pool connection.
    pub pool_connected: bool,
    /// Human-readable connection state (e.g. `connected`, `restarting`).
    pub connection_state: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolMinerDetectResult {
    pub found: bool,
    /// True when the running veriumd answered `poolminerdetect` successfully.
    pub rpc_ready: bool,
    pub sidecar_found: bool,
    pub path: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolMinerMemoryLimits {
    /// Auto-adjust / `-t 0` recommendation (from cpuminer `--tune` when available).
    pub max_safe_threads: u32,
    /// Manual slider ceiling (P-logical count on hybrid CPUs, else logical − 1).
    pub max_manual_threads: u32,
    pub scratchpad_mib: u32,
    pub total_ram_mib: u64,
    pub available_ram_mib: u64,
    pub uses_sidecar: bool,
}

#[derive(Deserialize)]
struct RpcPoolMinerInfo {
    running: bool,
    hashrate_hm: f64,
    worker: String,
    backend: String,
    last_log_line: String,
    threads: u32,
}

#[derive(Deserialize)]
struct RpcPoolMinerDetect {
    found: bool,
    backend: Option<String>,
}

/// Logical CPUs left for OS, WebView, veriumd, and Tauri while mining.
const UI_RESERVE_LOGICAL_CPUS: u32 = 2;

fn logical_cpu_count() -> u32 {
    std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(2)
        .max(1)
}

fn ui_reserve_logical_cpus(logical: u32) -> u32 {
    UI_RESERVE_LOGICAL_CPUS.max((logical / 10).min(4))
}

fn pool_cpu_thread_ceiling() -> u32 {
    let logical = logical_cpu_count();
    logical
        .saturating_sub(ui_reserve_logical_cpus(logical))
        .max(1)
}

fn apply_ui_thread_reserve(threads: u32) -> u32 {
    threads.min(pool_cpu_thread_ceiling()).max(1)
}

async fn cpuminer_recommended_threads_async(binary: Option<std::path::PathBuf>) -> u32 {
    tokio::task::spawn_blocking(move || cpuminer_recommended_threads(binary.as_deref()))
        .await
        .unwrap_or(1)
}

fn probe_system_memory_mib() -> (u64, u64) {
    let mut sys = System::new();
    sys.refresh_memory();
    let total = sys.total_memory() / (1024 * 1024);
    let available = sys.available_memory() / (1024 * 1024);
    (total, available)
}

async fn verium_rpc(state: &AppState) -> AppResult<crate::rpc::RpcClient> {
    state.rpc_client(CoinId::Verium).await
}

fn parse_pool_status(value: Value) -> AppResult<PoolMinerStatus> {
    let raw: RpcPoolMinerInfo = serde_json::from_value(value)
        .map_err(|e| AppError::other(format!("getpoolminerinfo parse error: {e}")))?;
    Ok(PoolMinerStatus {
        running: raw.running,
        hashrate_hm: raw.hashrate_hm,
        worker: raw.worker,
        last_log_line: raw.last_log_line,
        backend: if raw.backend.is_empty() {
            "native".into()
        } else {
            raw.backend
        },
        active_threads: raw.threads,
        accepted_shares: 0,
        rejected_shares: 0,
        pool_connected: raw.running,
        connection_state: if raw.running {
            "running".into()
        } else {
            "stopped".into()
        },
    })
}

fn supervisor_snapshot_to_status(
    snap: mining_supervisor::SupervisorSnapshot,
) -> PoolMinerStatus {
    PoolMinerStatus {
        running: snap.running,
        hashrate_hm: snap.hashrate_hm,
        worker: snap.worker,
        last_log_line: snap.last_log_line,
        backend: "sidecar".into(),
        active_threads: snap.threads,
        accepted_shares: snap.accepted,
        rejected_shares: snap.rejected,
        pool_connected: snap.pool_connected,
        connection_state: snap.connection_state,
    }
}

fn rpc_method_missing(err: &AppError) -> bool {
    matches!(
        err,
        AppError::Rpc { code, message }
            if *code == -32601
                || message.contains("not found")
                || message.contains("poolminer")
    )
}

/// Stop pool miner everywhere (sidecar + in-process); used during wallet shutdown.
pub async fn stop_pool_miner_rpc(state: &AppState) -> AppResult<()> {
    mining_supervisor::stop().await;
    let Ok(client) = verium_rpc(state).await else {
        return Ok(());
    };
    let _ = client.call::<Value>("poolminerstop", json!([])).await;
    Ok(())
}

pub async fn pool_miner_status_rpc(state: &AppState) -> AppResult<PoolMinerStatus> {
    if mining_supervisor::is_active().await {
        return Ok(supervisor_snapshot_to_status(
            mining_supervisor::snapshot().await,
        ));
    }
    let Ok(client) = verium_rpc(state).await else {
        return Ok(PoolMinerStatus::default());
    };
    match client.call("getpoolminerinfo", json!([])).await {
        Ok(value) => parse_pool_status(value),
        Err(e) if rpc_method_missing(&e) => Ok(PoolMinerStatus::default()),
        Err(_) => Ok(PoolMinerStatus::default()),
    }
}

static BUNDLED_POOL_DETECT: Lazy<Option<PoolMinerDetectResult>> =
    Lazy::new(bundled_pool_miner_detect_uncached);

fn bundled_pool_miner_detect() -> Option<PoolMinerDetectResult> {
    BUNDLED_POOL_DETECT.clone()
}

fn bundled_pool_miner_detect_uncached() -> Option<PoolMinerDetectResult> {
    let path = resolve_daemon_binary(CoinId::Verium)?;
    if !binary_supports_native_pool_mining(&path) {
        return None;
    }
    Some(PoolMinerDetectResult {
        found: true,
        rpc_ready: false,
        sidecar_found: true,
        path: Some(path.display().to_string()),
        source: "native-bundled".into(),
    })
}

pub async fn pool_miner_detect_rpc(state: &AppState) -> AppResult<PoolMinerDetectResult> {
    // Prefer the dedicated cpuminer sidecar when its binary is bundled.
    if let Some(path) = mining_supervisor::resolve_cpuminer_binary() {
        return Ok(PoolMinerDetectResult {
            found: true,
            rpc_ready: false,
            sidecar_found: true,
            path: Some(path.display().to_string()),
            source: "sidecar".into(),
        });
    }
    match verium_rpc(state).await {
        Ok(client) => match client.call("poolminerdetect", json!([])).await {
            Ok(value) => {
                let raw: RpcPoolMinerDetect = serde_json::from_value(value).map_err(|e| {
                    AppError::other(format!("poolminerdetect parse error: {e}"))
                })?;
                let backend = raw.backend.unwrap_or_else(|| "native".into());
                Ok(PoolMinerDetectResult {
                    found: raw.found,
                    rpc_ready: true,
                    sidecar_found: raw.found,
                    path: None,
                    source: backend,
                })
            }
            Err(e) if rpc_method_missing(&e) => Ok(bundled_pool_miner_detect().unwrap_or(
                PoolMinerDetectResult {
                    found: false,
                    rpc_ready: false,
                    sidecar_found: false,
                    path: None,
                    source: "none".into(),
                },
            )),
            Err(_) => Ok(bundled_pool_miner_detect().unwrap_or(PoolMinerDetectResult {
                found: false,
                rpc_ready: false,
                sidecar_found: false,
                path: None,
                source: "none".into(),
            })),
        },
        Err(_) => Ok(bundled_pool_miner_detect().unwrap_or(PoolMinerDetectResult {
            found: false,
            rpc_ready: false,
            sidecar_found: false,
            path: None,
            source: "unreachable".into(),
        })),
    }
}

pub async fn pool_miner_memory_limits_rpc(_state: &AppState) -> AppResult<PoolMinerMemoryLimits> {
    let (total_ram_mib, available_ram_mib) = probe_system_memory_mib();
    let uses_sidecar = mining_supervisor::sidecar_available();
    if uses_sidecar {
        let scratchpad_mib = cpuminer_scratchpad_mib(true);
        let binary = mining_supervisor::resolve_cpuminer_binary();
        let max_safe_threads = apply_ui_thread_reserve(
            cpuminer_recommended_threads_async(binary).await,
        );
        let topo = probe_topo();
        let max_manual_threads = apply_ui_thread_reserve(
            if topo.performance_cpus > 0 {
                topo.performance_cpus
            } else {
                pool_cpu_thread_ceiling()
            }
            .max(max_safe_threads),
        );
        return Ok(PoolMinerMemoryLimits {
            max_safe_threads,
            max_manual_threads,
            scratchpad_mib,
            total_ram_mib,
            available_ram_mib,
            uses_sidecar: true,
        });
    }
    // In-process MinGW path uses a much smaller lazily-faulted scratchpad.
    let scratchpad_mib: u32 = 128;
    let cpu_ceiling = pool_cpu_thread_ceiling();
    let ram_ceiling = (available_ram_mib / scratchpad_mib.max(1) as u64).max(1) as u32;
    let max_safe_threads = apply_ui_thread_reserve(cpu_ceiling.min(ram_ceiling).max(1));
    Ok(PoolMinerMemoryLimits {
        max_safe_threads,
        max_manual_threads: max_safe_threads,
        scratchpad_mib,
        total_ram_mib,
        available_ram_mib,
        uses_sidecar: false,
    })
}

#[tauri::command]
pub async fn pool_miner_detect(state: State<'_, AppState>) -> AppResult<PoolMinerDetectResult> {
    assert_verium(CoinId::Verium)?;
    pool_miner_detect_rpc(state.inner()).await
}

#[tauri::command]
pub async fn pool_miner_memory_limits(
    state: State<'_, AppState>,
) -> AppResult<PoolMinerMemoryLimits> {
    assert_verium(CoinId::Verium)?;
    pool_miner_memory_limits_rpc(state.inner()).await
}

#[tauri::command]
pub async fn pool_miner_status(state: State<'_, AppState>) -> AppResult<PoolMinerStatus> {
    assert_verium(CoinId::Verium)?;
    pool_miner_status_rpc(state.inner()).await
}

#[tauri::command]
pub async fn pool_miner_log_lines(
    state: State<'_, AppState>,
    max_lines: Option<usize>,
) -> AppResult<Vec<String>> {
    assert_verium(CoinId::Verium)?;
    let cap = max_lines.unwrap_or(120).max(1);
    if mining_supervisor::is_active().await {
        return Ok(mining_supervisor::log_lines(cap).await);
    }
    let st = pool_miner_status_rpc(state.inner()).await?;
    if st.last_log_line.is_empty() {
        return Ok(Vec::new());
    }
    Ok(vec![st.last_log_line].into_iter().take(cap).collect())
}

#[tauri::command]
pub async fn pool_miner_stop(state: State<'_, AppState>) -> AppResult<()> {
    assert_verium(CoinId::Verium)?;
    stop_pool_miner_rpc(state.inner()).await
}

#[tauri::command]
pub async fn pool_miner_start(
    state: State<'_, AppState>,
    config: PoolMinerStartConfig,
) -> AppResult<()> {
    assert_verium(CoinId::Verium)?;
    let password = config.password.as_deref().unwrap_or("x");
    let sidecar_binary = mining_supervisor::resolve_cpuminer_binary();
    let (auto_ceiling, manual_ceiling) = if let Some(binary) = sidecar_binary.clone() {
        let recommended = apply_ui_thread_reserve(
            cpuminer_recommended_threads_async(Some(binary)).await,
        );
        let topo = probe_topo();
        let manual = apply_ui_thread_reserve(
            if topo.performance_cpus > 0 {
                topo.performance_cpus
            } else {
                pool_cpu_thread_ceiling()
            }
            .max(recommended),
        );
        (recommended, manual)
    } else {
        let cap = pool_cpu_thread_ceiling();
        (cap, cap)
    };
    let threads = config.threads.max(1).min(manual_ceiling);
    if threads > auto_ceiling {
        tracing::info!(
            "pool miner: {} threads exceeds auto recommendation {} (cpuminer may warn about bandwidth)",
            threads,
            auto_ceiling
        );
    }

    // Prefer the dedicated cpuminer sidecar (MSVC SIMD, isolated from the node).
    if let Some(binary) = sidecar_binary {
        // Stop in-process solo mining first; the sidecar owns the CPU when pool mining.
        if let Ok(client) = verium_rpc(state.inner()).await {
            let _ = client.call::<Value>("minerstop", json!([])).await;
        }
        mining_supervisor::start(mining_supervisor::RunConfig {
            binary,
            stratum_url: config.stratum_url.trim().to_string(),
            backup_url: config.backup_url.as_ref().and_then(|s| {
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    Some(t.to_string())
                }
            }),
            username: config.username.trim().to_string(),
            password: password.to_string(),
            threads,
        })
        .await
        .map_err(AppError::other)?;
        // Surface an immediate launch failure (bad binary, etc.) to the UI.
        mining_supervisor::verify_started()
            .await
            .map_err(AppError::other)?;
        return Ok(());
    }

    // Fallback: in-process veriumd pool miner.
    let params = json!([
        threads,
        config.stratum_url.trim(),
        config.username.trim(),
        password,
    ]);
    let client = verium_rpc(state.inner()).await?;
    let _: Value = client.call("poolminerstart", params).await?;
    Ok(())
}
