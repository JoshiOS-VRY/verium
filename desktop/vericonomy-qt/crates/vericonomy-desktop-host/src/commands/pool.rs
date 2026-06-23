//! Pool CPU miner — cpuminer sidecar + in-process veriumd fallback.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::coin::CoinId;
use crate::context::AppContext;
use crate::cpuminer_topo::{cpuminer_recommended_threads, cpuminer_scratchpad_mib};
use crate::daemon_binary::{binary_supports_native_pool_mining, resolve_daemon_binary};
use crate::error::{HostError, HostResult};
use crate::mining_supervisor;
use crate::model::EarnState;
use crate::rpc::RpcClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolMinerStartConfig {
    pub stratum_url: String,
    pub username: String,
    pub password: Option<String>,
    pub threads: u32,
    #[serde(default)]
    pub backup_url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolMinerStatus {
    pub running: bool,
    pub hashrate_hm: f64,
    pub worker: String,
    pub last_log_line: String,
    pub backend: String,
    pub active_threads: u32,
    pub accepted_shares: u64,
    pub rejected_shares: u64,
    pub pool_connected: bool,
    pub connection_state: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolMinerDetectResult {
    pub found: bool,
    pub rpc_ready: bool,
    pub sidecar_found: bool,
    pub path: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolMinerMemoryLimits {
    pub max_safe_threads: u32,
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

const MINING_UI_RESERVE_CORES: u32 = 1;

fn assert_verium(coin: CoinId) -> HostResult<()> {
    if coin != CoinId::Verium {
        return Err(HostError::other("pool mining is Verium-only"));
    }
    Ok(())
}

fn logical_cpu_count() -> u32 {
    std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(2)
        .max(1)
}

fn pool_cpu_thread_ceiling() -> u32 {
    logical_cpu_count()
        .saturating_sub(MINING_UI_RESERVE_CORES)
        .max(1)
}

fn apply_ui_thread_reserve(threads: u32) -> u32 {
    threads.min(pool_cpu_thread_ceiling()).max(1)
}

async fn cpuminer_recommended_threads_async(binary: Option<PathBuf>) -> u32 {
    tokio::task::spawn_blocking(move || cpuminer_recommended_threads(binary.as_deref()))
        .await
        .unwrap_or(1)
}

fn probe_system_memory_mib() -> (u64, u64) {
    let mut sys = sysinfo::System::new();
    sys.refresh_memory();
    let total = sys.total_memory() / (1024 * 1024);
    let available = sys.available_memory() / (1024 * 1024);
    (total, available)
}

fn parse_pool_status(value: Value) -> HostResult<PoolMinerStatus> {
    let raw: RpcPoolMinerInfo = serde_json::from_value(value)
        .map_err(|e| HostError::other(format!("getpoolminerinfo parse error: {e}")))?;
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

fn rpc_method_missing(err: &HostError) -> bool {
    match err {
        HostError::Rpc { code, message } => {
            *code == -32601
                || message.contains("not found")
                || message.contains("poolminer")
        }
        _ => false,
    }
}

pub async fn stop_pool_miner(ctx: &AppContext) -> HostResult<()> {
    mining_supervisor::stop().await;
    stop_in_process_miners(ctx, CoinId::Verium).await;
    Ok(())
}

/// Stop veriumd solo + in-process pool miners so only one backend submits shares.
async fn stop_in_process_miners(ctx: &AppContext, coin: CoinId) {
    if let Some(ep) = ctx.endpoint(coin) {
        let client = RpcClient::new(ctx.http(), &ep);
        let _ = client.call("minerstop", json!([])).await;
        let _ = client.call("poolminerstop", json!([])).await;
    }
    ctx.set_earn(coin, EarnState::default());
}

pub async fn pool_miner_status(ctx: &AppContext, coin: CoinId) -> HostResult<PoolMinerStatus> {
    assert_verium(coin)?;
    if mining_supervisor::is_active().await {
        return Ok(supervisor_snapshot_to_status(
            mining_supervisor::snapshot().await,
        ));
    }
    let Some(ep) = ctx.endpoint(coin) else {
        return Ok(PoolMinerStatus::default());
    };
    let client = RpcClient::new(ctx.http(), &ep);
    match client.call("getpoolminerinfo", json!([])).await {
        Ok(value) => parse_pool_status(value),
        Err(e) if rpc_method_missing(&e) => Ok(PoolMinerStatus::default()),
        Err(_) => Ok(PoolMinerStatus::default()),
    }
}

fn bundled_pool_miner_detect() -> Option<PoolMinerDetectResult> {
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

pub async fn pool_miner_detect(ctx: &AppContext, coin: CoinId) -> HostResult<PoolMinerDetectResult> {
    assert_verium(coin)?;
    if let Some(path) = mining_supervisor::resolve_cpuminer_binary() {
        return Ok(PoolMinerDetectResult {
            found: true,
            rpc_ready: false,
            sidecar_found: true,
            path: Some(path.display().to_string()),
            source: "sidecar".into(),
        });
    }
    if let Some(ep) = ctx.endpoint(coin) {
        let client = RpcClient::new(ctx.http(), &ep);
        match client.call("poolminerdetect", json!([])).await {
            Ok(value) => {
                let raw: RpcPoolMinerDetect = serde_json::from_value(value).map_err(|e| {
                    HostError::other(format!("poolminerdetect parse error: {e}"))
                })?;
                let backend = raw.backend.unwrap_or_else(|| "native".into());
                return Ok(PoolMinerDetectResult {
                    found: raw.found,
                    rpc_ready: true,
                    sidecar_found: raw.found,
                    path: None,
                    source: backend,
                });
            }
            Err(e) if rpc_method_missing(&e) => {}
            Err(_) => {}
        }
    }
    Ok(bundled_pool_miner_detect().unwrap_or(PoolMinerDetectResult {
        found: false,
        rpc_ready: false,
        sidecar_found: false,
        path: None,
        source: "none".into(),
    }))
}

pub async fn pool_miner_memory_limits(_ctx: &AppContext, coin: CoinId) -> HostResult<PoolMinerMemoryLimits> {
    assert_verium(coin)?;
    let (total_ram_mib, available_ram_mib) = probe_system_memory_mib();
    let uses_sidecar = mining_supervisor::sidecar_available();
    if uses_sidecar {
        let scratchpad_mib = cpuminer_scratchpad_mib(true);
        let binary = mining_supervisor::resolve_cpuminer_binary();
        let max_safe_threads =
            apply_ui_thread_reserve(cpuminer_recommended_threads_async(binary).await);
        let cpu_ceiling = pool_cpu_thread_ceiling();
        let max_manual_threads = cpu_ceiling.max(max_safe_threads);
        return Ok(PoolMinerMemoryLimits {
            max_safe_threads,
            max_manual_threads,
            scratchpad_mib,
            total_ram_mib,
            available_ram_mib,
            uses_sidecar: true,
        });
    }
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

pub async fn pool_miner_log_lines(
    ctx: &AppContext,
    coin: CoinId,
    max_lines: Option<usize>,
) -> HostResult<Vec<String>> {
    assert_verium(coin)?;
    let cap = max_lines.unwrap_or(120).max(1);
    if mining_supervisor::is_active().await {
        return Ok(mining_supervisor::log_lines(cap).await);
    }
    let st = pool_miner_status(ctx, coin).await?;
    if st.last_log_line.is_empty() {
        return Ok(Vec::new());
    }
    Ok(vec![st.last_log_line].into_iter().take(cap).collect())
}

pub async fn pool_miner_start(
    ctx: &AppContext,
    coin: CoinId,
    config: PoolMinerStartConfig,
) -> HostResult<()> {
    assert_verium(coin)?;
    let password = config.password.as_deref().unwrap_or("x");
    let sidecar_binary = mining_supervisor::resolve_cpuminer_binary();
    let (auto_ceiling, manual_ceiling) = if let Some(binary) = sidecar_binary.clone() {
        let recommended =
            apply_ui_thread_reserve(cpuminer_recommended_threads_async(Some(binary)).await);
        let manual = pool_cpu_thread_ceiling().max(recommended);
        (recommended, manual)
    } else {
        let cap = pool_cpu_thread_ceiling();
        (cap, cap)
    };
    let threads = config.threads.max(1).min(manual_ceiling);
    let _ = auto_ceiling;

    if let Some(binary) = sidecar_binary {
        stop_in_process_miners(ctx, coin).await;
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
        .map_err(HostError::other)?;
        mining_supervisor::verify_started()
            .await
            .map_err(HostError::other)?;
        return Ok(());
    }

    let Some(ep) = ctx.endpoint(coin) else {
        return Err(HostError::other("Verium node RPC not available"));
    };
    mining_supervisor::stop().await;
    let client = RpcClient::new(ctx.http(), &ep);
    let params = json!([
        threads,
        config.stratum_url.trim(),
        config.username.trim(),
        password,
    ]);
    let _: Value = client.call("poolminerstart", params).await?;
    Ok(())
}
