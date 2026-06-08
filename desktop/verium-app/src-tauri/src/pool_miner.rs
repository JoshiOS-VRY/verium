//! Pool CPU miner via veriumd in-process Stratum (`poolminerstart` / `getpoolminerinfo`).

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sysinfo::System;
use tauri::State;

use crate::coin_profile::{assert_verium, CoinId};
use crate::daemon::{binary_supports_native_pool_mining, resolve_daemon_binary};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolMinerStartConfig {
    pub stratum_url: String,
    pub username: String,
    pub password: Option<String>,
    pub threads: u32,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolMinerStatus {
    pub running: bool,
    /// Local hashrate in H/m (matches solo mining display).
    pub hashrate_hm: f64,
    pub worker: String,
    pub last_log_line: String,
    /// `native` — hashing inside veriumd.
    pub backend: String,
    pub active_threads: u32,
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
    pub max_safe_threads: u32,
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

fn pool_cpu_thread_ceiling() -> u32 {
    std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(2)
        .saturating_sub(1)
        .max(1)
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
    })
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

/// Stop pool miner through veriumd (used during wallet shutdown).
pub async fn stop_pool_miner_rpc(state: &AppState) -> AppResult<()> {
    let Ok(client) = verium_rpc(state).await else {
        return Ok(());
    };
    let _ = client.call::<Value>("poolminerstop", json!([])).await;
    Ok(())
}

pub async fn pool_miner_status_rpc(state: &AppState) -> AppResult<PoolMinerStatus> {
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

pub async fn pool_miner_memory_limits_rpc(state: &AppState) -> AppResult<PoolMinerMemoryLimits> {
    let (total_ram_mib, available_ram_mib) = probe_system_memory_mib();
    let max_safe_threads = pool_cpu_thread_ceiling();
    Ok(PoolMinerMemoryLimits {
        max_safe_threads,
        scratchpad_mib: 128,
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
    let st = pool_miner_status_rpc(state.inner()).await?;
    let cap = max_lines.unwrap_or(120).max(1);
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
    let threads = config.threads.max(1).min(pool_cpu_thread_ceiling());
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
