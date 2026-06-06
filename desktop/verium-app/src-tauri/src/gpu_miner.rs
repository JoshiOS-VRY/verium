//! Optional experimental GPU miner sidecar (off by default).

use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::error::{AppError, AppResult};
use crate::memory_telemetry::{track_background_task_end, track_background_task_start};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuMinerConfig {
    pub enabled: bool,
    pub binary_path: Option<String>,
    pub pool_url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuMinerStatus {
    pub running: bool,
    pub hashrate: f64,
    pub last_log_line: String,
}

pub struct GpuMinerHandle {
    child: Mutex<Option<Child>>,
    status: Mutex<GpuMinerStatus>,
    cancel: Arc<AtomicBool>,
    tasks: Mutex<Vec<JoinHandle<()>>>,
}

impl GpuMinerHandle {
    pub fn new() -> Self {
        Self {
            child: Mutex::new(None),
            status: Mutex::new(GpuMinerStatus::default()),
            cancel: Arc::new(AtomicBool::new(true)),
            tasks: Mutex::new(Vec::new()),
        }
    }

    pub async fn status(&self) -> GpuMinerStatus {
        self.status.lock().await.clone()
    }

    async fn abort_tasks(&self) {
        self.cancel.store(true, Ordering::Release);
        let mut tasks = self.tasks.lock().await;
        for handle in tasks.drain(..) {
            handle.abort();
        }
    }

    pub async fn stop(&self) -> AppResult<()> {
        self.abort_tasks().await;
        let mut guard = self.child.lock().await;
        if let Some(mut child) = guard.take() {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
        let mut st = self.status.lock().await;
        st.running = false;
        Ok(())
    }

    pub async fn start(
        &self,
        cfg: &GpuMinerConfig,
        rpc_url: &str,
        rpc_user: &str,
        rpc_pass: &str,
    ) -> AppResult<()> {
        if !cfg.enabled {
            return Err(AppError::other("GPU miner is disabled (experimental opt-in)"));
        }
        let bin = cfg
            .binary_path
            .as_ref()
            .map(PathBuf::from)
            .filter(|p| p.exists())
            .ok_or_else(|| AppError::other("GPU miner binary path not configured or missing"))?;

        self.stop().await?;

        let mut cmd = Command::new(&bin);
        cmd.arg("--url").arg(rpc_url);
        cmd.arg("--userpass").arg(format!("{rpc_user}:{rpc_pass}"));
        cmd.arg("--coinbase-addr=local");
        if let Some(pool) = &cfg.pool_url {
            cmd.arg("--pool").arg(pool);
        }
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| AppError::other(format!("GPU miner spawn failed: {e}")))?;
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        {
            let mut st = self.status.lock().await;
            st.running = true;
            st.last_log_line.clear();
            st.hashrate = 0.0;
        }
        *self.child.lock().await = Some(child);

        self.cancel.store(false, Ordering::Release);
        let status = Arc::new(Mutex::new(self.status.lock().await.clone()));
        let cancel = Arc::clone(&self.cancel);

        let mut tasks = self.tasks.lock().await;
        if let Some(out) = stdout {
            tasks.push(spawn_log_reader(
                Arc::clone(&status),
                Arc::clone(&cancel),
                out,
            ));
        }
        if let Some(err) = stderr {
            tasks.push(spawn_log_reader(
                Arc::clone(&status),
                Arc::clone(&cancel),
                err,
            ));
        }

        Ok(())
    }
}

fn spawn_log_reader(
    status: Arc<Mutex<GpuMinerStatus>>,
    cancel: Arc<AtomicBool>,
    stream: impl tokio::io::AsyncRead + Unpin + Send + 'static,
) -> JoinHandle<()> {
    track_background_task_start();
    tokio::spawn(async move {
        let _guard = TaskGuard;
        let reader = BufReader::new(stream);
        let mut lines = reader.lines();
        while !cancel.load(Ordering::Acquire) {
            let next = lines.next_line().await;
            let Ok(Some(line)) = next else { break };
            let mut st = status.lock().await;
            st.last_log_line = line.clone();
            if let Some(hr) = parse_hashrate(&line) {
                st.hashrate = hr;
            }
        }
        let mut st = status.lock().await;
        st.running = false;
    })
}

struct TaskGuard;

impl Drop for TaskGuard {
    fn drop(&mut self) {
        track_background_task_end();
    }
}

fn parse_hashrate(line: &str) -> Option<f64> {
    for token in line.split_whitespace() {
        if let Ok(v) = token.parse::<f64>() {
            if line.to_lowercase().contains("hash") || line.contains("H/s") || line.contains("H/m")
            {
                return Some(v);
            }
        }
    }
    None
}

#[tauri::command]
pub async fn gpu_miner_status(handle: tauri::State<'_, GpuMinerHandle>) -> AppResult<GpuMinerStatus> {
    Ok(handle.status().await)
}

#[tauri::command]
pub async fn gpu_miner_stop(handle: tauri::State<'_, GpuMinerHandle>) -> AppResult<()> {
    handle.stop().await
}

#[tauri::command]
pub async fn gpu_miner_start(
    handle: tauri::State<'_, GpuMinerHandle>,
    config: GpuMinerConfig,
    rpc_url: String,
    rpc_user: String,
    rpc_pass: String,
) -> AppResult<()> {
    handle
        .start(&config, &rpc_url, &rpc_user, &rpc_pass)
        .await
}
