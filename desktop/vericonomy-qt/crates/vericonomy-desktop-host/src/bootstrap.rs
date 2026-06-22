//! Chain bootstrap import — port of Tauri `bootstrap.rs` for the Qt host.

use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use chrono::{Duration as ChronoDuration, Utc};
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;
use zip::read::ZipArchive;

use crate::bootstrap_session::{self, is_cancelled};
use crate::chain_layout::{
    bootstrap_chain_datadir, chain_snapshot_needs_reindex, promote_root_chain_data_for_unified,
    promote_subdir_chain_data_for_legacy, validate_bootstrap_staging,
};
use crate::coin::CoinId;
use crate::context::AppContext;
use crate::daemon_config::{load_daemon_config, verium_uses_legacy_flat, DaemonConfig};
use crate::daemon_manager::{
    force_stop_native_daemon, free_rpc_port, manager, pids_listening_on_port,
    start_managed_daemon, stop_managed_daemon,
};
use crate::error::{HostError, HostResult};
use crate::rpc::RpcClient;

const USER_AGENT: &str = "Vericonomy-Desktop/0.1";
const ZIP_LOCAL_MAGIC: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];
pub const PROGRESS_EVENT: &str = "bootstrap-progress";
pub const BOOTSTRAP_CANCELLED: &str = "Bootstrap cancelled by user.";

const MIN_BOOTSTRAP_BYTES: u64 = 1_000_000;
const DOWNLOAD_READ_TIMEOUT: Duration = Duration::from_secs(180);

const STOP_END: f64 = 5.0;
const RESOLVE_END: f64 = 7.0;
const DOWNLOAD_END: f64 = 67.0;
const VALIDATE_END: f64 = 70.0;
const EXTRACT_END: f64 = 95.0;
const APPLY_END: f64 = 98.0;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapProgress {
    pub coin: String,
    pub phase: String,
    pub percent: f64,
    pub phase_percent: Option<f64>,
    pub message: String,
    pub downloaded_bytes: Option<u64>,
    pub total_bytes: Option<u64>,
    pub extracted_files: Option<u64>,
    pub total_files: Option<u64>,
    pub source_url: Option<String>,
    pub eta_seconds: Option<u64>,
    pub cancellable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BootstrapResult {
    pub success: bool,
    pub message: String,
    pub restart_hint: Option<String>,
}

struct PhaseRateTracker {
    started: Instant,
    smoothed_rate: f64,
}

impl PhaseRateTracker {
    fn new() -> Self {
        Self {
            started: Instant::now(),
            smoothed_rate: 0.0,
        }
    }

    fn eta_seconds(
        &mut self,
        completed: u64,
        total: Option<u64>,
        min_elapsed_secs: f64,
    ) -> Option<u64> {
        let Some(total) = total.filter(|t| *t > completed) else {
            return None;
        };
        let elapsed = self.started.elapsed().as_secs_f64();
        if elapsed < min_elapsed_secs || completed == 0 {
            return None;
        }
        let instant_rate = completed as f64 / elapsed;
        self.smoothed_rate = if self.smoothed_rate <= 0.0 {
            instant_rate
        } else {
            self.smoothed_rate * 0.85 + instant_rate * 0.15
        };
        if self.smoothed_rate < 1.0 {
            return None;
        }
        Some(((total - completed) as f64 / self.smoothed_rate).ceil().max(1.0) as u64)
    }
}

fn is_cancellable_phase(phase: &str) -> bool {
    matches!(phase, "stopping" | "resolving" | "downloading")
}

pub fn is_bootstrap_cancelled(err: &HostError) -> bool {
    matches!(err, HostError::Other(msg) if msg.as_str() == BOOTSTRAP_CANCELLED)
}

struct BootstrapReporter<'a> {
    ctx: &'a AppContext,
    coin: CoinId,
    last_emit: Instant,
}

impl<'a> BootstrapReporter<'a> {
    fn new(ctx: &'a AppContext, coin: CoinId) -> Self {
        Self {
            ctx,
            coin,
            last_emit: Instant::now() - Duration::from_secs(1),
        }
    }

    fn lerp(start: f64, end: f64, phase_percent: f64) -> f64 {
        start + (end - start) * (phase_percent / 100.0).clamp(0.0, 1.0)
    }

    fn emit(&mut self, payload: BootstrapProgress, force: bool) {
        let now = Instant::now();
        if !force && now.duration_since(self.last_emit) < Duration::from_millis(200) {
            return;
        }
        self.last_emit = now;
        if let Ok(value) = serde_json::to_value(&payload) {
            self.ctx.bridge().emit(PROGRESS_EVENT, value);
        }
    }

    fn emit_phase(
        &mut self,
        phase: &str,
        percent: f64,
        phase_percent: Option<f64>,
        message: impl Into<String>,
        eta_seconds: Option<u64>,
        force: bool,
    ) {
        self.emit(
            BootstrapProgress {
                coin: self.coin.as_str().to_string(),
                phase: phase.into(),
                percent: percent.clamp(0.0, 100.0),
                phase_percent,
                message: message.into(),
                downloaded_bytes: None,
                total_bytes: None,
                extracted_files: None,
                total_files: None,
                source_url: None,
                eta_seconds,
                cancellable: is_cancellable_phase(phase),
            },
            force,
        );
    }

    fn stopping(&mut self, binary_name: &str) {
        self.emit_phase(
            "stopping",
            STOP_END * 0.5,
            Some(50.0),
            format!("Stopping {binary_name} before replacing chain data…"),
            None,
            true,
        );
    }

    fn resolving(&mut self) {
        self.emit_phase(
            "resolving",
            Self::lerp(STOP_END, RESOLVE_END, 50.0),
            Some(50.0),
            format!(
                "Finding the latest {} bootstrap on files.vericonomy.com…",
                self.coin.display_name()
            ),
            None,
            true,
        );
    }

    fn resolved(&mut self, url: &str) {
        self.emit(
            BootstrapProgress {
                coin: self.coin.as_str().to_string(),
                phase: "resolving".into(),
                percent: RESOLVE_END,
                phase_percent: Some(100.0),
                message: format!("Using bootstrap archive: {url}"),
                downloaded_bytes: None,
                total_bytes: None,
                extracted_files: None,
                total_files: None,
                source_url: Some(url.to_string()),
                eta_seconds: None,
                cancellable: true,
            },
            true,
        );
    }

    fn using_local(&mut self, path: &Path) {
        self.emit(
            BootstrapProgress {
                coin: self.coin.as_str().to_string(),
                phase: "local".into(),
                percent: RESOLVE_END,
                phase_percent: Some(100.0),
                message: format!("Using local bootstrap archive: {}", path.display()),
                downloaded_bytes: None,
                total_bytes: std::fs::metadata(path).ok().map(|m| m.len()),
                extracted_files: None,
                total_files: None,
                source_url: None,
                eta_seconds: None,
                cancellable: false,
            },
            true,
        );
    }

    fn downloading(
        &mut self,
        downloaded: u64,
        total: Option<u64>,
        source_url: Option<&str>,
        eta_seconds: Option<u64>,
        force: bool,
    ) {
        let phase_percent = total.map(|t| {
            if t == 0 {
                0.0
            } else {
                (downloaded as f64 / t as f64) * 100.0
            }
        });
        let overall = phase_percent
            .map(|p| Self::lerp(RESOLVE_END, DOWNLOAD_END, p))
            .unwrap_or(Self::lerp(RESOLVE_END, DOWNLOAD_END, 0.0));
        let message = match total {
            Some(total) if total > 0 => format!(
                "Downloading chain snapshot… {} of {}",
                format_bytes(downloaded),
                format_bytes(total)
            ),
            _ => format!(
                "Downloading chain snapshot… {} received",
                format_bytes(downloaded)
            ),
        };
        self.emit(
            BootstrapProgress {
                coin: self.coin.as_str().to_string(),
                phase: "downloading".into(),
                percent: overall,
                phase_percent,
                message,
                downloaded_bytes: Some(downloaded),
                total_bytes: total,
                extracted_files: None,
                total_files: None,
                source_url: source_url.map(str::to_string),
                eta_seconds,
                cancellable: true,
            },
            force,
        );
    }

    fn validating(&mut self) {
        self.emit_phase(
            "validating",
            Self::lerp(DOWNLOAD_END, VALIDATE_END, 50.0),
            Some(50.0),
            "Validating downloaded archive…",
            None,
            true,
        );
    }

    fn extracting(
        &mut self,
        extracted: u64,
        total: Option<u64>,
        indeterminate: bool,
        eta_seconds: Option<u64>,
        force: bool,
    ) {
        let phase_percent = if indeterminate {
            None
        } else {
            total.map(|t| {
                if t == 0 {
                    0.0
                } else {
                    (extracted as f64 / t as f64) * 100.0
                }
            })
        };
        let overall = phase_percent
            .map(|p| Self::lerp(VALIDATE_END, EXTRACT_END, p))
            .unwrap_or(Self::lerp(VALIDATE_END, EXTRACT_END, 35.0));
        let message = match (indeterminate, total) {
            (true, _) => {
                "Extracting blocks and chainstate (this may take several minutes)…".into()
            }
            (_, Some(total)) if total > 0 => {
                format!("Extracting blocks and chainstate… {extracted} / {total} files")
            }
            _ => format!("Extracting blocks and chainstate… {extracted} files"),
        };
        self.emit(
            BootstrapProgress {
                coin: self.coin.as_str().to_string(),
                phase: "extracting".into(),
                percent: overall,
                phase_percent,
                message,
                downloaded_bytes: None,
                total_bytes: None,
                extracted_files: Some(extracted),
                total_files: total,
                source_url: None,
                eta_seconds,
                cancellable: false,
            },
            force,
        );
    }

    fn applying(&mut self, step: u8) {
        let phase_percent = match step {
            0 => 20.0,
            1 => 60.0,
            _ => 100.0,
        };
        let message: String = match step {
            0 => "Replacing existing blocks/ directory…".into(),
            1 => "Replacing existing chainstate/ directory…".into(),
            _ => "Finalizing chain data in your datadir…".into(),
        };
        self.emit_phase(
            "applying",
            Self::lerp(EXTRACT_END, APPLY_END, phase_percent),
            Some(phase_percent),
            message,
            None,
            true,
        );
    }

    fn restarting(&mut self, binary_name: &str) {
        self.emit_phase(
            "restarting",
            Self::lerp(APPLY_END, 100.0, 50.0),
            Some(50.0),
            format!("Restarting {binary_name} with the new chain data…"),
            None,
            true,
        );
    }

    fn done(&mut self, message: impl Into<String>) {
        self.emit_phase("done", 100.0, Some(100.0), message, None, true);
    }

    fn failed(&mut self, message: impl Into<String>) {
        self.emit_phase("error", 0.0, None, message, None, true);
    }

    fn cancelled(&mut self) {
        self.emit_phase("cancelled", 0.0, None, BOOTSTRAP_CANCELLED, None, true);
    }
}

fn format_bytes(n: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if n >= GB {
        format!("{:.2} GB", n as f64 / GB as f64)
    } else if n >= MB {
        format!("{:.1} MB", n as f64 / MB as f64)
    } else if n >= KB {
        format!("{:.0} KB", n as f64 / KB as f64)
    } else {
        format!("{n} B")
    }
}

fn ensure_not_cancelled(cancel: &AtomicBool) -> HostResult<()> {
    if is_cancelled(cancel) {
        return Err(HostError::other(BOOTSTRAP_CANCELLED));
    }
    Ok(())
}

async fn cancellable_sleep(duration: Duration, cancel: &AtomicBool) -> bool {
    let started = Instant::now();
    while started.elapsed() < duration {
        if is_cancelled(cancel) {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    false
}

fn is_test_network(cfg: &DaemonConfig) -> bool {
    cfg.chain.contains("test") || cfg.chain.contains("binary")
}

pub async fn import_bootstrap(
    ctx: &AppContext,
    coin: CoinId,
    local_path: Option<PathBuf>,
) -> HostResult<BootstrapResult> {
    let cfg = load_daemon_config(coin)?;
    if is_test_network(&cfg) {
        return Err(HostError::other(
            "Bootstrap import is not available on the Binary Chain (binarytest) network. \
             Switch to Mainnet in Settings to import the official snapshot.",
        ));
    }

    let cancel = bootstrap_session::begin_async(coin).await;
    let mut reporter = BootstrapReporter::new(ctx, coin);
    let result = import_bootstrap_inner(ctx, coin, &cancel, local_path, &mut reporter).await;
    bootstrap_session::end_async(coin).await;
    if let Err(ref e) = result {
        if is_bootstrap_cancelled(e) {
            reporter.cancelled();
        } else {
            reporter.failed(e.to_string());
        }
    }
    result
}

pub fn cancel_bootstrap(coin: CoinId) {
    bootstrap_session::request_cancel(coin);
}

async fn import_bootstrap_inner(
    ctx: &AppContext,
    coin: CoinId,
    cancel: &AtomicBool,
    local_path: Option<PathBuf>,
    reporter: &mut BootstrapReporter<'_>,
) -> HostResult<BootstrapResult> {
    let cfg = load_daemon_config(coin)?;
    let datadir = bootstrap_chain_datadir(coin, &cfg);
    let binary_name = coin.binary_base();

    reporter.stopping(binary_name);
    stop_daemon_for_bootstrap(ctx, cancel).await;
    ensure_not_cancelled(cancel)?;

    let _ = promote_root_chain_data_for_unified(coin, &cfg)?;
    if coin == CoinId::Verium && verium_uses_legacy_flat(&cfg) {
        let _ = promote_subdir_chain_data_for_legacy(coin, &cfg)?;
    }

    let stale_zip = datadir.join(format!("bootstrap_{}.zip", coin.symbol()));
    if stale_zip.is_file() {
        let _ = std::fs::remove_file(&stale_zip);
    }

    let (archive_path, temp_download) =
        resolve_bootstrap_archive(ctx, coin, cancel, local_path, reporter).await?;
    validate_zip_download(&archive_path)?;
    reporter.validating();

    extract_bootstrap_zip(&archive_path, &datadir, reporter)?;
    apply_bootstrap(&datadir, reporter)?;

    if temp_download {
        let _ = std::fs::remove_file(&archive_path);
    }

    let result = finish_restart(ctx, coin, &cfg, binary_name, reporter).await?;
    reporter.done(&result.message);
    Ok(result)
}

async fn stop_daemon_for_bootstrap(ctx: &AppContext, cancel: &AtomicBool) {
    for target in CoinId::all() {
        let _ = stop_managed_daemon(ctx, target).await;
        if is_cancelled(cancel) {
            return;
        }
    }
    for target in CoinId::all() {
        if let Ok(cfg) = load_daemon_config(target) {
            clear_datadir_lock_file(&bootstrap_chain_datadir(target, &cfg));
        }
    }
    cancellable_sleep(Duration::from_secs(1), cancel).await;
}

fn clear_datadir_lock_file(datadir: &Path) {
    let lock = datadir.join(".lock");
    if lock.is_file() {
        let _ = std::fs::remove_file(&lock);
    }
}

async fn resolve_bootstrap_archive(
    ctx: &AppContext,
    coin: CoinId,
    cancel: &AtomicBool,
    local_path: Option<PathBuf>,
    reporter: &mut BootstrapReporter<'_>,
) -> HostResult<(PathBuf, bool)> {
    if let Some(path) = resolve_local_bootstrap(coin, local_path)? {
        reporter.using_local(&path);
        return Ok((path, false));
    }

    let client = build_bootstrap_http_client()?;
    reporter.resolving();
    let url = resolve_bootstrap_url(coin, &client, cancel).await?;
    reporter.resolved(&url);

    let temp_zip = temp_bootstrap_path(coin);
    download_bootstrap_zip(ctx, &client, &url, &temp_zip, cancel, reporter).await?;
    Ok((temp_zip, true))
}

fn build_bootstrap_http_client() -> HostResult<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .connect_timeout(Duration::from_secs(60))
        .read_timeout(DOWNLOAD_READ_TIMEOUT)
        .tcp_keepalive(Duration::from_secs(60))
        .http1_only()
        .build()
        .map_err(Into::into)
}

fn resolve_local_bootstrap(
    coin: CoinId,
    explicit: Option<PathBuf>,
) -> HostResult<Option<PathBuf>> {
    if let Some(path) = explicit {
        if is_valid_bootstrap_zip(&path) {
            return Ok(Some(path));
        }
        return Err(HostError::other(format!(
            "Local bootstrap is missing or not a valid zip archive: {}",
            path.display()
        )));
    }

    for key in local_bootstrap_env_keys(coin) {
        if let Ok(raw) = std::env::var(key) {
            let path = PathBuf::from(raw.trim());
            if is_valid_bootstrap_zip(&path) {
                return Ok(Some(path));
            }
        }
    }

    if let Some(home) = dirs::home_dir() {
        for path in [
            home.join("Downloads").join(local_bootstrap_filename(coin)),
            home.join("Desktop").join(local_bootstrap_filename(coin)),
        ] {
            if is_valid_bootstrap_zip(&path) {
                return Ok(Some(path));
            }
        }
    }

    Ok(None)
}

fn local_bootstrap_env_keys(coin: CoinId) -> &'static [&'static str] {
    match coin {
        CoinId::Verium => &["VERIUM_BOOTSTRAP_LOCAL"],
        CoinId::Vericoin => &["VERICOIN_BOOTSTRAP_LOCAL", "VERICOIND_BOOTSTRAP_LOCAL"],
    }
}

fn local_bootstrap_filename(coin: CoinId) -> String {
    match coin {
        CoinId::Verium => "verium-bootstrap.zip".into(),
        CoinId::Vericoin => "vericoin-bootstrap.zip".into(),
    }
}

fn is_valid_bootstrap_zip(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    if meta.len() < MIN_BOOTSTRAP_BYTES {
        return false;
    }
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    let mut header = [0u8; 4];
    file.read_exact(&mut header).is_ok() && header == ZIP_LOCAL_MAGIC
}

fn temp_bootstrap_path(coin: CoinId) -> PathBuf {
    std::env::temp_dir().join(format!(
        "{}-bootstrap-{}.zip",
        coin.as_str(),
        Uuid::new_v4()
    ))
}

async fn resolve_bootstrap_url(
    coin: CoinId,
    client: &reqwest::Client,
    cancel: &AtomicBool,
) -> HostResult<String> {
    let base = coin.bootstrap_cdn_base();
    let archive_name = match coin {
        CoinId::Verium => "verium-bootstrap.zip",
        CoinId::Vericoin => "vericoin-bootstrap.zip",
    };
    let canonical = format!("{base}/{archive_name}");
    let mut candidates = vec![canonical.clone()];

    let today = Utc::now().date_naive();
    let dated_prefix = match coin {
        CoinId::Verium => "verium-bootstrap",
        CoinId::Vericoin => "vericoin-bootstrap",
    };
    for days_back in 0..14 {
        let date = today - ChronoDuration::days(days_back);
        candidates.push(format!("{base}/{dated_prefix}-{date}.zip"));
    }

    for url in candidates {
        if is_cancelled(cancel) {
            return Err(HostError::other(BOOTSTRAP_CANCELLED));
        }
        if url_available(client, &url).await? {
            return Ok(url);
        }
    }

    Err(HostError::other(format!(
        "No bootstrap archive found on files.vericonomy.com for {} (tried canonical and recent dated zips).",
        coin.display_name()
    )))
}

async fn url_available(client: &reqwest::Client, url: &str) -> HostResult<bool> {
    match client
        .get(url)
        .header("Range", "bytes=0-0")
        .header("Accept-Encoding", "identity")
        .send()
        .await
    {
        Ok(resp) => Ok(resp.status().is_success() || resp.status().as_u16() == 206),
        Err(_) => Ok(false),
    }
}

fn map_download_error(url: &str, err: reqwest::Error) -> HostError {
    let detail = err.to_string();
    let hint = if detail.contains("decoding response body") || detail.contains("connection") {
        " The CDN connection dropped during download — use a local zip or retry to resume."
    } else {
        ""
    };
    HostError::other(format!(
        "Failed downloading bootstrap from {url}: {detail}.{hint}"
    ))
}

async fn download_bootstrap_zip(
    _ctx: &AppContext,
    client: &reqwest::Client,
    url: &str,
    target: &Path,
    cancel: &AtomicBool,
    reporter: &mut BootstrapReporter<'_>,
) -> HostResult<()> {
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut downloaded = if target.is_file() {
        std::fs::metadata(target).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    reporter.downloading(downloaded, None, Some(url), None, true);
    ensure_not_cancelled(cancel)?;

    let mut request = client
        .get(url)
        .header("Accept-Encoding", "identity");
    if downloaded > 0 {
        request = request.header("Range", format!("bytes={downloaded}-"));
    }

    let mut resp = request.send().await.map_err(|e| map_download_error(url, e))?;
    let status = resp.status();

    if downloaded > 0 && status == reqwest::StatusCode::OK {
        downloaded = 0;
        let _ = std::fs::remove_file(target);
    }

    if !status.is_success() {
        return Err(HostError::other(format!(
            "Download failed: server responded with HTTP {status} for {url}"
        )));
    }

    let expected_len = if status.as_u16() == 206 {
        resp.content_length().map(|n| downloaded + n)
    } else {
        resp.content_length()
    };

    let mut rate = PhaseRateTracker::new();
    let mut file = if downloaded == 0 {
        std::fs::File::create(target)?
    } else {
        OpenOptions::new().append(true).open(target)?
    };

    while let Some(chunk) = resp.chunk().await.map_err(|e| map_download_error(url, e))? {
        if is_cancelled(cancel) {
            drop(file);
            return Err(HostError::other(BOOTSTRAP_CANCELLED));
        }
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;
        let eta = rate.eta_seconds(downloaded, expected_len, 2.0);
        let force = downloaded < 512 * 1024
            || downloaded.is_multiple_of(512 * 1024)
            || expected_len.is_some_and(|t| downloaded >= t);
        reporter.downloading(downloaded, expected_len, Some(url), eta, force);
    }
    file.sync_all()?;

    if let Some(expected) = expected_len {
        if downloaded != expected {
            return Err(HostError::other(format!(
                "Download incomplete: expected {expected} bytes, got {downloaded}. \
                 Retry to resume, or choose a local bootstrap zip."
            )));
        }
    }

    reporter.downloading(downloaded, Some(downloaded), Some(url), Some(0), true);
    Ok(())
}

fn validate_zip_download(path: &Path) -> HostResult<()> {
    let mut file = std::fs::File::open(path)?;
    let mut header = [0u8; 4];
    file.read_exact(&mut header)?;
    if header != ZIP_LOCAL_MAGIC {
        let preview = String::from_utf8_lossy(&header);
        let _ = std::fs::remove_file(path);
        return Err(HostError::other(format!(
            "Download is not a valid zip archive (header {preview:?}). \
             The CDN may have returned an error page — try again in a minute."
        )));
    }
    Ok(())
}

fn count_zip_files(archive: &mut ZipArchive<std::fs::File>) -> HostResult<u64> {
    let mut count = 0u64;
    for i in 0..archive.len() {
        let entry = archive
            .by_index(i)
            .map_err(|e| HostError::other(format!("zip entry {i}: {e}")))?;
        if entry.is_dir() || entry.name().ends_with('/') {
            continue;
        }
        count += 1;
    }
    Ok(count)
}

fn extract_bootstrap_zip(
    zip_path: &Path,
    datadir: &Path,
    reporter: &mut BootstrapReporter<'_>,
) -> HostResult<()> {
    let staging = datadir.join("bootstrap");
    if staging.exists() {
        std::fs::remove_dir_all(&staging)?;
    }
    std::fs::create_dir_all(&staging)?;

    let file = std::fs::File::open(zip_path)?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| HostError::other(format!("invalid zip: {e}")))?;

    let total_files = count_zip_files(&mut archive)?;
    reporter.extracting(0, Some(total_files), false, None, true);

    let first_name = (0..archive.len())
        .find_map(|i| archive.by_index(i).ok().map(|f| f.name().to_string()));
    let top_level_blocks = first_name
        .as_deref()
        .is_some_and(|n| n.starts_with("blocks/") || n == "blocks");

    let mut extracted = 0u64;
    let mut rate = PhaseRateTracker::new();
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| HostError::other(format!("zip entry {i}: {e}")))?;
        let raw_name = entry.name().to_string();
        if raw_name.ends_with('/') {
            continue;
        }

        let relative = if top_level_blocks {
            PathBuf::from(&raw_name)
        } else if let Some(stripped) = raw_name.strip_prefix("bootstrap/") {
            PathBuf::from(stripped)
        } else {
            PathBuf::from(&raw_name)
        };

        let out_path = staging.join(relative);
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)?;
            continue;
        }

        let mut out = std::fs::File::create(&out_path)?;
        std::io::copy(&mut entry, &mut out)?;
        extracted += 1;
        let eta = rate.eta_seconds(extracted, Some(total_files), 3.0);
        let force = extracted == 1
            || extracted == total_files
            || extracted.is_multiple_of(25);
        reporter.extracting(extracted, Some(total_files), false, eta, force);
    }

    reporter.extracting(extracted, Some(total_files), false, Some(0), true);
    Ok(())
}

fn remove_dir_all_with_retry(path: &Path, label: &str) -> HostResult<()> {
    for attempt in 0..5 {
        match std::fs::remove_dir_all(path) {
            Ok(()) => return Ok(()),
            Err(e) if e.raw_os_error() == Some(32) && attempt + 1 < 5 => {
                force_stop_native_daemon(CoinId::Verium);
                force_stop_native_daemon(CoinId::Vericoin);
                std::thread::sleep(Duration::from_secs(2));
            }
            Err(_) if !path.exists() => return Ok(()),
            Err(e) => {
                return Err(HostError::other(format!(
                    "could not remove {label} at {}: {e}. \
                     Quit the wallet, stop veriumd/vericoind in Task Manager, then retry bootstrap.",
                    path.display()
                )));
            }
        }
    }
    Err(HostError::other(format!(
        "could not remove {label} at {} after several attempts (file in use). \
         Quit the wallet, stop veriumd/vericoind in Task Manager, then retry bootstrap.",
        path.display()
    )))
}

fn apply_bootstrap(datadir: &Path, reporter: &mut BootstrapReporter<'_>) -> HostResult<()> {
    let staging = datadir.join("bootstrap");
    validate_bootstrap_staging(&staging)?;

    reporter.applying(0);
    clear_datadir_lock_file(datadir);

    for name in ["blocks", "chainstate"] {
        let target = datadir.join(name);
        if target.exists() {
            remove_dir_all_with_retry(&target, name)?;
        }
    }

    for (idx, name) in ["blocks", "chainstate"].iter().enumerate() {
        let from = staging.join(name);
        let target = datadir.join(name);
        std::fs::rename(&from, &target).map_err(|e| {
            HostError::other(format!(
                "could not install bootstrap {name}/ into {}: {e}",
                datadir.display()
            ))
        })?;
        reporter.applying((idx + 1) as u8);
    }

    if chain_snapshot_needs_reindex(datadir) {
        return Err(HostError::other(
            "Bootstrap chainstate failed to install (blocks present but index is empty). \
             Quit the wallet, delete any veriumd processes, and retry import.",
        ));
    }

    let peers = staging.join("peers.dat");
    if peers.is_file() {
        let _ = std::fs::copy(&peers, datadir.join("peers.dat"));
    }
    let _ = std::fs::remove_dir_all(&staging);
    reporter.applying(2);
    Ok(())
}

async fn wait_for_rpc_port_free(port: u16, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if pids_listening_on_port(port).is_empty() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

async fn poll_chain_height_after_restart(
    ctx: &AppContext,
    coin: CoinId,
    cfg: &DaemonConfig,
    max_wait: Duration,
) -> Option<u64> {
    let deadline = Instant::now() + max_wait;
    let mut best = 0u64;
    while Instant::now() < deadline {
        tokio::time::sleep(Duration::from_secs(2)).await;
        if ctx.endpoint(coin).is_none() {
            crate::daemon_manager::sync_ctx_endpoint(ctx, coin, cfg);
        }
        let Some(ep) = ctx.endpoint(coin) else {
            continue;
        };
        let client = RpcClient::new(ctx.http(), &ep);
        if let Ok(info) = client.call("getblockchaininfo", json!([])).await {
            if let Some(blocks) = info.get("blocks").and_then(|v| v.as_u64()) {
                best = best.max(blocks);
                if blocks > 10_000 {
                    return Some(blocks);
                }
            }
        }
    }
    if best > 0 { Some(best) } else { None }
}

fn bootstrap_success_message(binary_name: &str, blocks: Option<u64>) -> String {
    match blocks {
        Some(height) if height > 10_000 => format!(
            "Bootstrap applied and {binary_name} was restarted at block #{height}. \
             Your node will continue syncing the remaining blocks from the network."
        ),
        Some(height) => format!(
            "Bootstrap applied and {binary_name} was restarted (currently at block #{height}). \
             The node will keep syncing from the network."
        ),
        None => format!(
            "Bootstrap applied and {binary_name} was restarted. \
             Chain data is loading — block height should appear within a few minutes."
        ),
    }
}

async fn finish_restart(
    ctx: &AppContext,
    coin: CoinId,
    cfg: &DaemonConfig,
    binary_name: &str,
    reporter: &mut BootstrapReporter<'_>,
) -> HostResult<BootstrapResult> {
    reporter.restarting(binary_name);
    manager(coin).await.clear_tracking().await;

    tokio::time::sleep(Duration::from_millis(1500)).await;

    let restart_cfg = load_daemon_config(coin)?;
    let datadir = bootstrap_chain_datadir(coin, &restart_cfg);

    if crate::daemon_binary::resolve_daemon_binary(coin).is_some() {
        if let Some(ep) = ctx.endpoint(coin) {
            let client = RpcClient::new(ctx.http(), &ep);
            let _ = client.call("stop", json!([])).await;
        }
        manager(coin).await.force_kill_child().await;
        free_rpc_port(coin, &restart_cfg);
        force_stop_native_daemon(coin);
        wait_for_rpc_port_free(restart_cfg.rpc_port, Duration::from_secs(30)).await;

        if chain_snapshot_needs_reindex(&datadir) {
            return Ok(BootstrapResult {
                success: false,
                message: "Bootstrap did not install a complete chainstate. \
                          Quit the wallet, stop all veriumd processes, and try Download snapshot again."
                    .into(),
                restart_hint: None,
            });
        }

        match start_managed_daemon(ctx, coin).await {
            Ok(_) => {
                let blocks =
                    poll_chain_height_after_restart(ctx, coin, &restart_cfg, Duration::from_secs(90))
                        .await;
                return Ok(BootstrapResult {
                    success: true,
                    message: bootstrap_success_message(binary_name, blocks),
                    restart_hint: None,
                });
            }
            Err(e) => {
                return Ok(BootstrapResult {
                    success: true,
                    message: format!(
                        "Bootstrap applied, but automatic restart failed: {e}. Start {binary_name} manually."
                    ),
                    restart_hint: None,
                });
            }
        }
    }

    Ok(BootstrapResult {
        success: true,
        message: format!("Bootstrap applied. Start {binary_name} manually to continue."),
        restart_hint: None,
    })
}
