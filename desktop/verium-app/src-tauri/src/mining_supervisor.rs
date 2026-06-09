//! Pool-mining sidecar supervisor.
//!
//! Spawns the bundled cpuminer (veriumMiner) as a child process, monitors it via
//! its local TCP control API, keeps a ring buffer of log lines, and auto-restarts
//! on crash with exponential backoff. This decouples pool hashing from the veriumd
//! node process so node restarts and sync spikes do not interrupt mining, and so
//! Windows users get the MSVC SIMD miner instead of the throughput-1 MinGW path.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;
use serde::Serialize;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::time::{sleep, timeout};

const MAX_LOG_LINES: usize = 500;
const API_HOST: &str = "127.0.0.1";
/// Local control/telemetry port passed to cpuminer via `--api-bind`.
const API_PORT: u16 = 14048;
const POLL_INTERVAL: Duration = Duration::from_secs(5);
/// A run lasting longer than this is considered healthy; resets restart backoff.
const HEALTHY_RUN: Duration = Duration::from_secs(30);
const MAX_BACKOFF_SECS: u64 = 30;
/// Real cpuminer binaries are >1 MB; the fetch script writes a tiny stub on failure.
const MIN_REAL_BINARY_BYTES: u64 = 200_000;

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupervisorSnapshot {
    pub running: bool,
    /// Local hashrate in H/m (matches solo mining display).
    pub hashrate_hm: f64,
    pub worker: String,
    pub threads: u32,
    pub accepted: u64,
    pub rejected: u64,
    pub pool_connected: bool,
    pub connection_state: String,
    pub last_log_line: String,
    pub restart_count: u32,
}

#[derive(Clone)]
pub struct RunConfig {
    pub binary: PathBuf,
    pub stratum_url: String,
    pub backup_url: Option<String>,
    pub username: String,
    pub password: String,
    pub threads: u32,
}

struct Inner {
    desired_running: bool,
    generation: u64,
    pid: Option<u32>,
    snapshot: SupervisorSnapshot,
    logs: VecDeque<String>,
}

impl Default for Inner {
    fn default() -> Self {
        Inner {
            desired_running: false,
            generation: 0,
            pid: None,
            snapshot: SupervisorSnapshot::default(),
            logs: VecDeque::with_capacity(MAX_LOG_LINES),
        }
    }
}

static SUPERVISOR: Lazy<Mutex<Inner>> = Lazy::new(|| Mutex::new(Inner::default()));

fn push_log(inner: &mut Inner, line: String) {
    if inner.logs.len() >= MAX_LOG_LINES {
        inner.logs.pop_front();
    }
    inner.snapshot.last_log_line = line.clone();
    inner.logs.push_back(line);
}

async fn still_current(generation: u64) -> bool {
    let inner = SUPERVISOR.lock().await;
    inner.desired_running && inner.generation == generation
}

/// Image/file name of the bundled cpuminer binary (e.g. `cpuminer-<triple>.exe`).
fn cpuminer_image_name() -> Option<String> {
    resolve_cpuminer_binary()?
        .file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

/// Best-effort kill of any bundled cpuminer instances left over from a previous
/// session (e.g. a wallet restart where `kill_on_drop` did not fire). Matches the
/// bundled binary's exact file name so a user's own standalone miner is untouched.
pub async fn kill_stray_miners() {
    let Some(image) = cpuminer_image_name() else {
        return;
    };
    #[cfg(target_os = "windows")]
    {
        let mut std_cmd = std::process::Command::new("taskkill");
        std_cmd.args(["/F", "/T", "/IM", &image]);
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        std_cmd.creation_flags(CREATE_NO_WINDOW);
        let _ = Command::from(std_cmd).output().await;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = Command::new("pkill").args(["-f", &image]).output().await;
    }
}

/// Start (or restart) the sidecar miner with the given config.
pub async fn start(cfg: RunConfig) -> Result<(), String> {
    stop().await;
    // Clear any orphaned instances so the API port is free and we don't double-mine.
    kill_stray_miners().await;

    let generation = {
        let mut inner = SUPERVISOR.lock().await;
        inner.desired_running = true;
        inner.generation += 1;
        inner.snapshot = SupervisorSnapshot {
            running: false,
            worker: cfg.username.clone(),
            threads: cfg.threads,
            connection_state: "starting".into(),
            ..Default::default()
        };
        push_log(
            &mut inner,
            format!("Starting pool miner sidecar ({} threads)", cfg.threads),
        );
        inner.generation
    };

    tokio::spawn(run_supervisor(generation, cfg));
    tokio::spawn(run_api_poller(generation));
    Ok(())
}

/// Stop the sidecar miner and invalidate its supervising tasks.
pub async fn stop() {
    let pid = {
        let mut inner = SUPERVISOR.lock().await;
        if !inner.desired_running && inner.pid.is_none() {
            return;
        }
        inner.desired_running = false;
        inner.generation += 1;
        inner.snapshot.running = false;
        inner.snapshot.hashrate_hm = 0.0;
        inner.snapshot.pool_connected = false;
        inner.snapshot.connection_state = "stopped".into();
        push_log(&mut inner, "Pool miner stopped".into());
        inner.pid.take()
    };
    if let Some(pid) = pid {
        kill_pid(pid).await;
    }
}

pub async fn snapshot() -> SupervisorSnapshot {
    SUPERVISOR.lock().await.snapshot.clone()
}

/// Wait briefly after a start and report an error only if the miner clearly failed
/// to launch (so transient "still connecting" states do not surface as errors).
pub async fn verify_started() -> Result<(), String> {
    sleep(Duration::from_millis(1500)).await;
    let inner = SUPERVISOR.lock().await;
    if inner.snapshot.running {
        return Ok(());
    }
    if inner.snapshot.connection_state == "error"
        || inner.snapshot.last_log_line.starts_with("Failed to launch")
    {
        let last = inner.snapshot.last_log_line.clone();
        return Err(if last.is_empty() {
            "Pool miner failed to launch".to_string()
        } else {
            last
        });
    }
    Ok(())
}

pub async fn is_active() -> bool {
    let inner = SUPERVISOR.lock().await;
    inner.desired_running || inner.snapshot.running
}

pub async fn log_lines(max: usize) -> Vec<String> {
    let inner = SUPERVISOR.lock().await;
    let total = inner.logs.len();
    let take = max.min(total);
    inner.logs.iter().skip(total - take).cloned().collect()
}

async fn run_supervisor(generation: u64, cfg: RunConfig) {
    let mut backoff: u64 = 1;

    loop {
        if !still_current(generation).await {
            break;
        }

        let started = Instant::now();
        let mut child = match spawn_miner(&cfg) {
            Ok(child) => child,
            Err(e) => {
                {
                    let mut inner = SUPERVISOR.lock().await;
                    push_log(&mut inner, format!("Failed to launch miner: {e}"));
                    inner.snapshot.connection_state = "error".into();
                }
                if !still_current(generation).await {
                    break;
                }
                sleep(Duration::from_secs(backoff)).await;
                backoff = (backoff * 2).min(MAX_BACKOFF_SECS);
                continue;
            }
        };

        let pid = child.id();
        {
            let mut inner = SUPERVISOR.lock().await;
            inner.pid = pid;
            inner.snapshot.running = true;
            inner.snapshot.connection_state = "connecting".into();
            push_log(&mut inner, format!("Miner started (pid {})", pid.unwrap_or(0)));
        }

        if let Some(out) = child.stdout.take() {
            tokio::spawn(read_stream(generation, out));
        }
        if let Some(err) = child.stderr.take() {
            tokio::spawn(read_stream(generation, err));
        }

        let status = child.wait().await;

        {
            let mut inner = SUPERVISOR.lock().await;
            inner.snapshot.running = false;
            inner.snapshot.hashrate_hm = 0.0;
            inner.snapshot.pool_connected = false;
            inner.pid = None;
            let code = match &status {
                Ok(s) => s
                    .code()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "signal".into()),
                Err(e) => format!("wait error: {e}"),
            };
            push_log(&mut inner, format!("Miner exited (status {code})"));
        }

        if !still_current(generation).await {
            break;
        }

        // Reset backoff after a healthy run; otherwise grow it to throttle crash loops.
        if started.elapsed() >= HEALTHY_RUN {
            backoff = 1;
        }
        {
            let mut inner = SUPERVISOR.lock().await;
            inner.snapshot.restart_count += 1;
            inner.snapshot.connection_state = "restarting".into();
            push_log(
                &mut inner,
                format!("Restarting miner in {backoff}s…"),
            );
        }
        sleep(Duration::from_secs(backoff)).await;
        backoff = (backoff * 2).min(MAX_BACKOFF_SECS);
    }

    let mut inner = SUPERVISOR.lock().await;
    if inner.generation == generation {
        inner.snapshot.running = false;
        if inner.snapshot.connection_state != "stopped" {
            inner.snapshot.connection_state = "stopped".into();
        }
    }
}

fn spawn_miner(cfg: &RunConfig) -> std::io::Result<tokio::process::Child> {
    let mut std_cmd = std::process::Command::new(&cfg.binary);
    std_cmd
        .arg("-o")
        .arg(&cfg.stratum_url)
        .arg("-u")
        .arg(&cfg.username)
        .arg("-p")
        .arg(&cfg.password)
        .arg("-t")
        .arg(cfg.threads.to_string())
        .arg("-b")
        .arg(format!("{API_HOST}:{API_PORT}"))
        .arg("--no-color")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(backup) = &cfg.backup_url {
        if !backup.is_empty() {
            std_cmd.arg("--backup-url").arg(backup);
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW — keep cpuminer headless under the wallet.
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        std_cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let mut cmd = Command::from(std_cmd);
    cmd.kill_on_drop(true);
    cmd.spawn()
}

async fn read_stream<R>(generation: u64, stream: R)
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut lines = BufReader::new(stream).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let mut inner = SUPERVISOR.lock().await;
        if inner.generation != generation {
            break;
        }
        let trimmed = line.trim_end().to_string();
        if !trimmed.is_empty() {
            push_log(&mut inner, trimmed);
        }
    }
}

async fn kill_pid(pid: u32) {
    #[cfg(target_os = "windows")]
    {
        let mut std_cmd = std::process::Command::new("taskkill");
        std_cmd.args(["/PID", &pid.to_string(), "/T", "/F"]);
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        std_cmd.creation_flags(CREATE_NO_WINDOW);
        let _ = Command::from(std_cmd).output().await;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .output()
            .await;
    }
}

#[derive(Default)]
struct ParsedSummary {
    hpm: Option<f64>,
    hpm_avg: Option<f64>,
    accepted: Option<u64>,
    rejected: Option<u64>,
    worker: Option<String>,
    pool_connected: Option<bool>,
}

fn parse_summary(s: &str) -> ParsedSummary {
    let mut out = ParsedSummary::default();
    for field in s.trim_end_matches('|').split(';') {
        let Some((key, value)) = field.split_once('=') else {
            continue;
        };
        match key {
            "HPM" => out.hpm = value.parse().ok(),
            "HPM_AVG60" => out.hpm_avg = value.parse().ok(),
            "ACC" => out.accepted = value.parse().ok(),
            "REJ" => out.rejected = value.parse().ok(),
            "WORKER" => out.worker = Some(value.to_string()),
            "POOL" => out.pool_connected = value.parse::<i32>().ok().map(|n| n != 0),
            _ => {}
        }
    }
    out
}

async fn query_summary() -> Option<String> {
    let addr = format!("{API_HOST}:{API_PORT}");
    let connect = timeout(Duration::from_secs(2), TcpStream::connect(&addr)).await;
    let mut stream = match connect {
        Ok(Ok(s)) => s,
        _ => return None,
    };
    if timeout(Duration::from_secs(2), stream.write_all(b"summary\n"))
        .await
        .ok()?
        .is_err()
    {
        return None;
    }
    let mut buf = Vec::with_capacity(1024);
    let read = timeout(Duration::from_secs(2), stream.read_to_end(&mut buf)).await;
    match read {
        Ok(Ok(_)) => {
            let text = String::from_utf8_lossy(&buf);
            Some(text.trim_end_matches('\0').to_string())
        }
        _ => None,
    }
}

async fn run_api_poller(generation: u64) {
    loop {
        if !still_current(generation).await {
            break;
        }
        if let Some(resp) = query_summary().await {
            let parsed = parse_summary(&resp);
            let mut inner = SUPERVISOR.lock().await;
            if inner.generation != generation {
                break;
            }
            if let Some(h) = parsed.hpm_avg.or(parsed.hpm) {
                inner.snapshot.hashrate_hm = h;
            }
            if let Some(a) = parsed.accepted {
                inner.snapshot.accepted = a;
            }
            if let Some(r) = parsed.rejected {
                inner.snapshot.rejected = r;
            }
            if let Some(w) = parsed.worker {
                if !w.is_empty() {
                    inner.snapshot.worker = w;
                }
            }
            if let Some(connected) = parsed.pool_connected {
                inner.snapshot.pool_connected = connected;
                inner.snapshot.connection_state =
                    if connected { "connected".into() } else { "connecting".into() };
            }
        }
        sleep(POLL_INTERVAL).await;
    }
}

fn is_real_binary(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|m| m.is_file() && m.len() > MIN_REAL_BINARY_BYTES)
        .unwrap_or(false)
}

/// Locate the bundled cpuminer sidecar, skipping the build placeholder stub.
///
/// Tauri `externalBin` ships `cpuminer{.exe}` next to the wallet executable.
/// Dev builds also keep `cpuminer-<triple>{.exe}` under `src-tauri/binaries/`.
pub fn resolve_cpuminer_binary() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let parent = exe.parent()?.to_path_buf();
    let mut candidates: Vec<PathBuf> = Vec::new();

    // Release bundle (same pattern as veriumd.exe in `detect_sidecar_binary`).
    candidates.push(if cfg!(target_os = "windows") {
        parent.join("cpuminer.exe")
    } else {
        parent.join("cpuminer")
    });

    if let Ok(triple) = std::env::var("TARGET") {
        let name = if cfg!(target_os = "windows") {
            format!("cpuminer-{triple}.exe")
        } else {
            format!("cpuminer-{triple}")
        };
        candidates.push(parent.join(&name));
    }

    let dirs = [
        parent.clone(),
        parent.join("binaries"),
        parent.join("..").join("..").join("binaries"),
        parent.join("..").join("..").join("..").join("binaries"),
    ];
    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            let fname = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let ext_ok = if cfg!(target_os = "windows") {
                fname.ends_with(".exe")
            } else {
                !fname.contains('.') || fname.ends_with(".bin")
            };
            if fname.starts_with("cpuminer-") && ext_ok && p.is_file() {
                candidates.push(p);
            }
        }
    }

    candidates.into_iter().find(|p| is_real_binary(p))
}

pub fn sidecar_available() -> bool {
    resolve_cpuminer_binary().is_some()
}
