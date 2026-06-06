//! veriumMiner / cpuminer Stratum sidecar (AVX2 SIMD — fastest pool path).

use std::collections::VecDeque;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::error::{AppError, AppResult};
use crate::memory_telemetry::{track_background_task_end, track_background_task_start};

const MAX_LOG_LINES: usize = 250;
const MAX_LOG_LINE_CHARS: usize = 512;

const CPUMINER_API_ADDR: &str = "127.0.0.1:4048";
const API_POLL_INTERVAL: Duration = Duration::from_secs(15);

const CPUMINER_BASE: &str = "cpuminer";
/// Tauri build placeholders are tiny; real veriumMiner releases are hundreds of KB+.
const MIN_CPUMINER_BYTES: u64 = 100_000;

fn is_real_cpuminer_binary(path: &PathBuf) -> bool {
    if !path.is_file() {
        return false;
    }
    std::fs::metadata(path)
        .map(|m| m.len() >= MIN_CPUMINER_BYTES)
        .unwrap_or(false)
}

pub fn detect_cpuminer_binary() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("CPUMINER_PATH") {
        let path = PathBuf::from(p);
        if is_real_cpuminer_binary(&path) {
            return Some(path);
        }
    }

    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.join(cpuminer_exe_name()));
            if let Ok(entries) = std::fs::read_dir(parent) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    let fname = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    if p.is_file()
                        && (fname.starts_with("cpuminer-") && fname.ends_with(".exe")
                            || fname == "cpuminer.exe")
                    {
                        candidates.push(p);
                    }
                }
            }
        }
    }

    for dir in binaries_search_dirs() {
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let fname = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if fname.starts_with("cpuminer-") && is_cpuminer_candidate(fname) && p.is_file() {
                    candidates.push(p);
                }
            }
        }
        candidates.push(dir.join(cpuminer_exe_name()));
    }

    if let Some(p) = which_cpuminer_on_path() {
        candidates.push(p);
    }

    candidates.into_iter().find(|p| is_real_cpuminer_binary(p))
}

fn cpuminer_exe_name() -> String {
    if cfg!(windows) {
        format!("{CPUMINER_BASE}.exe")
    } else {
        CPUMINER_BASE.to_string()
    }
}

fn is_cpuminer_candidate(fname: &str) -> bool {
    if cfg!(windows) {
        fname.ends_with(".exe")
    } else {
        !fname.contains('.') || fname.ends_with(".bin")
    }
}

fn binaries_search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            dirs.push(parent.join("binaries"));
            dirs.push(parent.join("..").join("..").join("..").join("binaries"));
            dirs.push(parent.join("..").join("..").join("binaries"));
        }
    }
    dirs.push(PathBuf::from("src-tauri/binaries"));
    dirs
}

fn which_cpuminer_on_path() -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    let names = if cfg!(windows) {
        vec!["cpuminer.exe", "cpuminer"]
    } else {
        vec!["cpuminer"]
    };
    for dir in std::env::split_paths(&path_var) {
        for name in &names {
            let p = dir.join(name);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

struct LogBuffer {
    lines: StdMutex<VecDeque<String>>,
}

impl Default for LogBuffer {
    fn default() -> Self {
        Self {
            lines: StdMutex::new(VecDeque::new()),
        }
    }
}

impl LogBuffer {
    fn clear(&self) {
        self.lines.lock().unwrap().clear();
    }

    fn push(&self, line: &str) {
        let clean = strip_ansi_escapes(line);
        let trimmed = clean.trim();
        if trimmed.is_empty() {
            return;
        }
        let bounded = if trimmed.chars().count() > MAX_LOG_LINE_CHARS {
            let mut out = String::with_capacity(MAX_LOG_LINE_CHARS + 1);
            for ch in trimmed.chars().take(MAX_LOG_LINE_CHARS) {
                out.push(ch);
            }
            out.push('…');
            out
        } else {
            trimmed.to_string()
        };
        let mut guard = self.lines.lock().unwrap();
        guard.push_back(bounded);
        while guard.len() > MAX_LOG_LINES {
            guard.pop_front();
        }
    }

    fn snapshot_tail(&self, max_lines: usize) -> Vec<String> {
        let guard = self.lines.lock().unwrap();
        let take = max_lines.max(1).min(guard.len());
        guard
            .iter()
            .skip(guard.len().saturating_sub(take))
            .cloned()
            .collect()
    }
}

pub struct CpuminerSidecar {
    child: Mutex<Option<Child>>,
    status: Arc<Mutex<SidecarStatus>>,
    logs: Arc<LogBuffer>,
    poll_cancel: Arc<AtomicBool>,
    tasks: Mutex<Vec<JoinHandle<()>>>,
    binary: PathBuf,
}

#[derive(Clone, Default)]
pub struct SidecarStatus {
    pub running: bool,
    pub hashrate_hm: f64,
    pub worker: String,
    pub last_log_line: String,
    pub threads: u32,
}

impl CpuminerSidecar {
    pub fn new(binary: PathBuf) -> Self {
        Self {
            child: Mutex::new(None),
            status: Arc::new(Mutex::new(SidecarStatus::default())),
            logs: Arc::new(LogBuffer::default()),
            poll_cancel: Arc::new(AtomicBool::new(true)),
            tasks: Mutex::new(Vec::new()),
            binary,
        }
    }

    pub async fn status(&self) -> SidecarStatus {
        self.status.lock().await.clone()
    }

    pub fn log_lines_tail(&self, max_lines: usize) -> Vec<String> {
        self.logs.snapshot_tail(max_lines)
    }

    async fn abort_tasks(&self) {
        self.poll_cancel.store(true, Ordering::Release);
        let mut tasks = self.tasks.lock().await;
        for handle in tasks.drain(..) {
            handle.abort();
        }
    }

    pub async fn stop(&self) {
        self.abort_tasks().await;
        let mut guard = self.child.lock().await;
        if let Some(mut child) = guard.take() {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
        let mut st = self.status.lock().await;
        st.running = false;
        st.hashrate_hm = 0.0;
    }

    pub async fn start(
        &self,
        stratum_url: &str,
        username: &str,
        password: &str,
        threads: u32,
    ) -> AppResult<()> {
        self.stop().await;
        self.logs.clear();

        let url = normalize_stratum_url(stratum_url);
        let mut std_cmd = std::process::Command::new(&self.binary);
        std_cmd
            .arg("-o")
            .arg(&url)
            .arg("-u")
            .arg(username)
            .arg("-p")
            .arg(password)
            .arg("-t")
            .arg(threads.max(1).to_string())
            .arg("-b")
            .arg(CPUMINER_API_ADDR)
            .arg("--status-interval")
            .arg("15")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            std_cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let mut cmd = Command::from(std_cmd);
        cmd.kill_on_drop(false);

        let mut child = cmd
            .spawn()
            .map_err(|e| AppError::other(format!("failed to spawn cpuminer: {e}")))?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        {
            let mut st = self.status.lock().await;
            st.running = true;
            st.worker = username.to_string();
            st.threads = threads.max(1);
            st.hashrate_hm = 0.0;
            st.last_log_line = format!(
                "veriumMiner started ({}) — {} thread(s)",
                self.binary.display(),
                threads
            );
        }
        self.logs.push(&format!(
            "veriumMiner started — {} thread(s), pool {}",
            threads.max(1),
            url
        ));

        *self.child.lock().await = Some(child);

        self.poll_cancel.store(false, Ordering::Release);
        let mut tasks = self.tasks.lock().await;
        tasks.push(spawn_api_poller(
            Arc::clone(&self.status),
            Arc::clone(&self.poll_cancel),
        ));

        if let Some(out) = stdout {
            tasks.push(spawn_log_reader(
                Arc::clone(&self.status),
                Arc::clone(&self.logs),
                out,
            ));
        }
        if let Some(err) = stderr {
            tasks.push(spawn_log_reader(
                Arc::clone(&self.status),
                Arc::clone(&self.logs),
                err,
            ));
        }

        Ok(())
    }
}

fn normalize_stratum_url(url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.starts_with("stratum+tcp://") || trimmed.starts_with("stratum://") {
        trimmed.to_string()
    } else if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("stratum+tcp://{trimmed}")
    }
}

fn spawn_log_reader(
    status: Arc<Mutex<SidecarStatus>>,
    logs: Arc<LogBuffer>,
    stream: impl tokio::io::AsyncRead + Unpin + Send + 'static,
) -> JoinHandle<()> {
    track_background_task_start();
    tokio::spawn(async move {
        let _guard = TaskGuard;
        let reader = BufReader::new(stream);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let clean = strip_ansi_escapes(&line);
            logs.push(&clean);
            let hr = parse_cpuminer_hashrate(&clean);
            let mut st = status.lock().await;
            st.last_log_line = clean;
            if let Some(hr) = hr {
                st.hashrate_hm = hr;
            }
        }
    })
}

fn spawn_api_poller(status: Arc<Mutex<SidecarStatus>>, cancel: Arc<AtomicBool>) -> JoinHandle<()> {
    track_background_task_start();
    tokio::spawn(async move {
        let _guard = TaskGuard;
        while !cancel.load(Ordering::Acquire) {
            if let Some(hpm) = fetch_cpuminer_hpm().await {
                let mut st = status.lock().await;
                st.hashrate_hm = hpm;
            }
            tokio::time::sleep(API_POLL_INTERVAL).await;
        }
    })
}

struct TaskGuard;

impl Drop for TaskGuard {
    fn drop(&mut self) {
        track_background_task_end();
    }
}

async fn fetch_cpuminer_hpm() -> Option<f64> {
    let mut stream = tokio::time::timeout(Duration::from_secs(2), TcpStream::connect(CPUMINER_API_ADDR))
        .await
        .ok()?
        .ok()?;
    stream.write_all(b"summary\n").await.ok()?;
    let mut buf = vec![0u8; 4096];
    let n = tokio::time::timeout(Duration::from_secs(2), stream.read(&mut buf))
        .await
        .ok()?
        .ok()?;
    if n == 0 {
        return None;
    }
    let text = String::from_utf8_lossy(&buf[..n]);
    parse_summary_hpm(&text).or_else(|| parse_json_hpm(&text))
}

/// veriumMiner monitoring API: `HPM=3590.51;...` (hashes per minute).
pub fn parse_summary_hpm(text: &str) -> Option<f64> {
    for part in text.split(';') {
        let part = part.trim();
        if let Some(raw) = part.strip_prefix("HPM=") {
            if let Ok(v) = raw.parse::<f64>() {
                if v.is_finite() && v > 0.0 {
                    return Some(v);
                }
            }
        }
    }
    None
}

fn parse_json_hpm(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    let value: serde_json::Value = serde_json::from_str(trimmed).ok()?;
    if let Some(hpm) = value.get("hashrate_hpm").and_then(|v| v.as_f64()) {
        if hpm.is_finite() && hpm > 0.0 {
            return Some(hpm);
        }
    }
    if let Some(hps) = value
        .get("hashrate_ema_60s")
        .or_else(|| value.get("hashrate_hps"))
        .and_then(|v| v.as_f64())
    {
        if hps.is_finite() && hps > 0.0 {
            return Some(hps * 60.0);
        }
    }
    None
}

/// Strip ANSI SGR sequences from veriumMiner structured logs.
pub fn strip_ansi_escapes(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.next_if_eq(&'[').is_some() {
                for ch in chars.by_ref() {
                    if ch.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

/// Parse legacy `Total: 1234.56 H/m` lines (fallback when API is unavailable).
pub fn parse_cpuminer_hashrate(line: &str) -> Option<f64> {
    let clean = strip_ansi_escapes(line);
    if let Some(hpm) = parse_summary_hpm(&clean) {
        return Some(hpm);
    }

    let lower = clean.to_lowercase();
    if lower.contains("total:") {
        for token in clean.split_whitespace() {
            if token.eq_ignore_ascii_case("total:") || token.eq_ignore_ascii_case("total") {
                continue;
            }
            if let Ok(v) = token.trim_end_matches(',').parse::<f64>() {
                if v.is_finite() && v > 100.0 {
                    return Some(v);
                }
            }
        }
    }

    if let Some(idx) = lower.find("h/m") {
        let before = &clean[..idx];
        if let Some(v) = last_positive_number(before) {
            if v > 100.0 {
                return Some(v);
            }
        }
    }

    None
}

fn last_positive_number(s: &str) -> Option<f64> {
    let mut last = None;
    for token in s.split_whitespace() {
        let trimmed = token
            .trim_matches(|c: char| !c.is_ascii_digit() && c != '.' && c != ',');
        if let Ok(v) = trimmed.replace(',', "").parse::<f64>() {
            if v.is_finite() && v > 0.0 {
                last = Some(v);
            }
        }
    }
    last
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_total_hm() {
        assert_eq!(
            parse_cpuminer_hashrate("Total: 5123.45 H/m"),
            Some(5123.45)
        );
    }

    #[test]
    fn parses_summary_api_hpm() {
        let sample = "NAME=veriumminer;CPUS=19;HPM=3590.51;HPM_AVG60=3592.80;";
        assert_eq!(parse_summary_hpm(sample), Some(3590.51));
    }

    #[test]
    fn ignores_thread_count_in_status_line() {
        assert_eq!(
            parse_cpuminer_hashrate("19 threads active, 3590 H/m"),
            Some(3590.0)
        );
    }

    #[test]
    fn strips_ansi_from_logs() {
        let raw = "\x1b[01;30m[2026-06-05 00:28:34]\x1b[0m \x1b[01;36mINFO\x1b[0m Pool difficulty";
        assert_eq!(
            strip_ansi_escapes(raw),
            "[2026-06-05 00:28:34] INFO Pool difficulty"
        );
    }

    #[test]
    fn parses_json_hps_as_hpm() {
        let raw = r#"{"hashrate_ema_60s":59.88,"threads":19}"#;
        assert_eq!(parse_json_hpm(raw), Some(59.88 * 60.0));
    }
}
