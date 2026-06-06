//! Pool CPU miner: veriumMiner/cpuminer sidecar (fastest) with native fallback.

use serde::{Deserialize, Serialize};
use sysinfo::System;
use tokio::sync::Mutex;
use verium_pool_miner::{
    parse_stratum_url, pool_memory_limits, EngineConfig, PoolMinerEngine, SystemMemory,
};

use crate::error::{AppError, AppResult};
use crate::pool_miner_sidecar::{detect_cpuminer_binary, CpuminerSidecar};

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
    /// `veriumMiner` (sidecar) or `native` (in-wallet fallback).
    pub backend: String,
    pub active_threads: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolMinerDetectResult {
    pub found: bool,
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
    /// When true, veriumMiner sidecar is used for hashing.
    pub uses_sidecar: bool,
}

enum ActiveBackend {
    None,
    Sidecar(CpuminerSidecar),
    Native(PoolMinerEngine),
}

pub struct PoolMinerHandle {
    backend: Mutex<ActiveBackend>,
}

fn probe_system_memory() -> SystemMemory {
    let mut sys = System::new();
    sys.refresh_memory();
    SystemMemory {
        total_bytes: sys.total_memory(),
        available_bytes: sys.available_memory(),
    }
}

/// In-wallet native fallback: one thread only (~128 MiB scratchpad). Full thread
/// count requires the veriumMiner sidecar (out-of-process).
const NATIVE_IN_PROCESS_MAX_THREADS: u32 = 1;

/// Logical CPUs − 1 so one core remains for the OS and wallet (sidecar only).
fn pool_cpu_thread_ceiling() -> u32 {
    std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(2)
        .saturating_sub(1)
        .max(1)
}

impl PoolMinerHandle {
    pub fn new() -> Self {
        Self {
            backend: Mutex::new(ActiveBackend::None),
        }
    }

    async fn stop_inner(&self) {
        let mut guard = self.backend.lock().await;
        match &mut *guard {
            ActiveBackend::Sidecar(s) => s.stop().await,
            ActiveBackend::Native(e) => e.stop(),
            ActiveBackend::None => {}
        }
        *guard = ActiveBackend::None;
    }

    pub async fn log_lines(&self, max_lines: usize) -> Vec<String> {
        let guard = self.backend.lock().await;
        match &*guard {
            ActiveBackend::Sidecar(s) => s.log_lines_tail(max_lines),
            ActiveBackend::Native(e) => {
                let st = e.status();
                if st.last_log_line.is_empty() {
                    Vec::new()
                } else {
                    vec![st.last_log_line]
                }
            }
            ActiveBackend::None => Vec::new(),
        }
    }

    pub async fn status(&self) -> PoolMinerStatus {
        let guard = self.backend.lock().await;
        match &*guard {
            ActiveBackend::Sidecar(s) => {
                let st = s.status().await;
                PoolMinerStatus {
                    running: st.running,
                    hashrate_hm: st.hashrate_hm,
                    worker: st.worker,
                    last_log_line: st.last_log_line,
                    backend: "veriumMiner".into(),
                    active_threads: st.threads,
                }
            }
            ActiveBackend::Native(e) => {
                let st = e.status();
                PoolMinerStatus {
                    running: st.running,
                    hashrate_hm: st.hashrate_hm,
                    worker: st.worker,
                    last_log_line: st.last_log_line,
                    backend: "native".into(),
                    active_threads: st.active_threads,
                }
            }
            ActiveBackend::None => PoolMinerStatus::default(),
        }
    }

    pub async fn stop(&self) -> AppResult<()> {
        self.stop_inner().await;
        Ok(())
    }

    pub async fn start(&self, cfg: &PoolMinerStartConfig) -> AppResult<()> {
        self.stop_inner().await;

        let password = cfg.password.as_deref().unwrap_or("x");
        let requested = cfg.threads.max(1);

        if let Some(bin) = detect_cpuminer_binary() {
            let cpu_ceiling = pool_cpu_thread_ceiling();
            let threads = requested.min(cpu_ceiling).max(1);
            let sidecar = CpuminerSidecar::new(bin);
            sidecar
                .start(
                    &cfg.stratum_url,
                    cfg.username.trim(),
                    password,
                    threads,
                )
                .await?;
            *self.backend.lock().await = ActiveBackend::Sidecar(sidecar);
            return Ok(());
        }

        // Native fallback when veriumMiner sidecar is unavailable.
        let threads = requested.min(NATIVE_IN_PROCESS_MAX_THREADS).max(1);
        let (host, port) = parse_stratum_url(&cfg.stratum_url).map_err(AppError::other)?;
        let engine = PoolMinerEngine::new();
        let engine_cfg = EngineConfig {
            host,
            port,
            username: cfg.username.trim().to_string(),
            password: password.to_string(),
            threads,
        };
        engine.start(engine_cfg).map_err(AppError::other)?;
        *self.backend.lock().await = ActiveBackend::Native(engine);
        Ok(())
    }
}

pub fn detect_pool_miner() -> PoolMinerDetectResult {
    if let Some(path) = detect_cpuminer_binary() {
        return PoolMinerDetectResult {
            found: true,
            sidecar_found: true,
            path: Some(path.display().to_string()),
            source: "veriumMiner".into(),
        };
    }
    PoolMinerDetectResult {
        found: true,
        sidecar_found: false,
        path: None,
        source: "native".into(),
    }
}

pub fn pool_miner_memory_limits_sync() -> PoolMinerMemoryLimits {
    let mem = probe_system_memory();
    let uses_sidecar = detect_cpuminer_binary().is_some();
    let cpu = if uses_sidecar {
        pool_cpu_thread_ceiling()
    } else {
        NATIVE_IN_PROCESS_MAX_THREADS
    };
    let limits = pool_memory_limits(mem, Some(cpu));
    PoolMinerMemoryLimits {
        max_safe_threads: limits.max_safe_threads,
        scratchpad_mib: limits.scratchpad_mib,
        total_ram_mib: limits.total_ram_mib,
        available_ram_mib: limits.available_ram_mib,
        uses_sidecar,
    }
}

#[tauri::command]
pub async fn pool_miner_detect() -> AppResult<PoolMinerDetectResult> {
    Ok(detect_pool_miner())
}

#[tauri::command]
pub async fn pool_miner_memory_limits() -> AppResult<PoolMinerMemoryLimits> {
    Ok(pool_miner_memory_limits_sync())
}

#[tauri::command]
pub async fn pool_miner_status(handle: tauri::State<'_, PoolMinerHandle>) -> AppResult<PoolMinerStatus> {
    Ok(handle.status().await)
}

#[tauri::command]
pub async fn pool_miner_log_lines(
    handle: tauri::State<'_, PoolMinerHandle>,
    max_lines: Option<usize>,
) -> AppResult<Vec<String>> {
    Ok(handle.log_lines(max_lines.unwrap_or(120)).await)
}

#[tauri::command]
pub async fn pool_miner_stop(handle: tauri::State<'_, PoolMinerHandle>) -> AppResult<()> {
    handle.stop().await
}

#[tauri::command]
pub async fn pool_miner_start(
    handle: tauri::State<'_, PoolMinerHandle>,
    config: PoolMinerStartConfig,
) -> AppResult<()> {
    handle.start(&config).await
}
