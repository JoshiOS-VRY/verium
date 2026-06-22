//! Daemon binary discovery — port of Tauri `daemon.rs` detection helpers.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::coin::CoinId;
use crate::config::app_config_base;
use crate::error::{HostError, HostResult};

const MIN_REAL_SIDECAR_BYTES: u64 = 100_000;
const POOL_MINER_DETECT_MARKER: &[u8] = b"poolminerdetect";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DaemonBinarySource {
    Sidecar,
    Env,
    AdjacentToApp,
    Path,
    SystemDefault,
    None,
}

#[derive(Debug, Clone, Serialize)]
pub struct DaemonBinaryStatus {
    pub found: bool,
    pub path: Option<String>,
    pub source: DaemonBinarySource,
    pub manageable: bool,
    pub runtime: String,
    pub coin: String,
    pub stub_sidecar: bool,
    pub missing_hint: Option<String>,
}

pub fn is_real_sidecar(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|m| m.len() >= MIN_REAL_SIDECAR_BYTES)
        .unwrap_or(false)
}

fn is_daemon_sidecar(path: &Path) -> bool {
    let Some(fname) = path.file_name().and_then(|s| s.to_str()) else {
        return false;
    };
    fname == "veriumd"
        || fname == "veriumd.exe"
        || fname == "vericoind"
        || fname == "vericoind.exe"
        || fname.starts_with("veriumd-")
        || fname.starts_with("vericoind-")
        || path.to_string_lossy().contains("binaries")
}

fn staged_sidecar_path(coin: CoinId) -> PathBuf {
    app_config_base().join("run").join(coin.binary_name())
}

fn is_staged_sidecar_path(path: &Path, coin: CoinId) -> bool {
    path == staged_sidecar_path(coin)
}

fn rank_sidecar_candidate(path: &Path, coin: CoinId) -> (bool, u64, u64) {
    let staged = is_staged_sidecar_path(path, coin);
    let mtime = path
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let size = path.metadata().map(|m| m.len()).unwrap_or(0);
    (!staged, mtime, size)
}

fn pick_best_sidecar(coin: CoinId, candidates: &[PathBuf]) -> Option<PathBuf> {
    candidates
        .iter()
        .max_by_key(|p| rank_sidecar_candidate(p, coin))
        .cloned()
}

pub fn binary_help_output(path: &Path) -> Option<String> {
    if !path.is_file() {
        return None;
    }
    let path = path.to_path_buf();
    std::thread::scope(|scope| {
        let (tx, rx) = std::sync::mpsc::channel();
        scope.spawn(move || {
            let mut cmd = std::process::Command::new(&path);
            cmd.arg("-help");
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                const CREATE_NO_WINDOW: u32 = 0x0800_0000;
                cmd.creation_flags(CREATE_NO_WINDOW);
            }
            let result = cmd.output().ok().map(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                format!("{stdout}{stderr}")
            });
            let _ = tx.send(result);
        });
        rx.recv_timeout(std::time::Duration::from_secs(8)).ok().flatten()
    })
}

pub fn binary_supports_unified_chain_selector(path: &Path, coin: CoinId) -> bool {
    let Some(help) = binary_help_output(path) else {
        return false;
    };
    match coin {
        CoinId::Verium => help.contains("-verium"),
        CoinId::Vericoin => help.contains("-vericoin"),
    }
}

fn file_identity(path: &Path) -> Option<(u64, u64)> {
    let meta = std::fs::metadata(path).ok()?;
    let size = meta.len();
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Some((size, mtime))
}

fn scan_file_for_marker(path: &Path, marker: &[u8]) -> bool {
    use std::io::Read;
    if marker.is_empty() {
        return false;
    }
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    let overlap = marker.len().saturating_sub(1);
    let mut carry = Vec::new();
    let mut chunk = [0u8; 256 * 1024];
    loop {
        let Ok(read) = file.read(&mut chunk) else {
            return false;
        };
        if read == 0 {
            return false;
        }
        let mut data = carry;
        data.extend_from_slice(&chunk[..read]);
        if data.windows(marker.len()).any(|w| w == marker) {
            return true;
        }
        carry = data[data.len().saturating_sub(overlap)..].to_vec();
    }
}

static POOL_MINING_MARKER_CACHE: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<PathBuf, (u64, u64, bool)>>,
> = std::sync::OnceLock::new();

pub fn binary_supports_native_pool_mining(path: &Path) -> bool {
    let Some((size, mtime)) = file_identity(path) else {
        return false;
    };
    let cache = POOL_MINING_MARKER_CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    if let Ok(guard) = cache.lock() {
        if let Some(&(cached_size, cached_mtime, supported)) = guard.get(path) {
            if cached_size == size && cached_mtime == mtime {
                return supported;
            }
        }
    }
    let supported = scan_file_for_marker(path, POOL_MINER_DETECT_MARKER);
    if let Ok(mut guard) = cache.lock() {
        guard.insert(path.to_path_buf(), (size, mtime, supported));
    }
    supported
}

fn pick_preferred_sidecar(coin: CoinId, mut candidates: Vec<PathBuf>) -> Option<PathBuf> {
    candidates.retain(|p| is_real_sidecar(p));
    if candidates.is_empty() {
        return None;
    }
    if coin == CoinId::Verium {
        let pool_native: Vec<PathBuf> = candidates
            .iter()
            .filter(|p| binary_supports_native_pool_mining(p))
            .cloned()
            .collect();
        if let Some(best) = pick_best_sidecar(coin, &pool_native) {
            return Some(best);
        }
        let mut legacy: Vec<PathBuf> = candidates
            .into_iter()
            .filter(|p| !binary_supports_unified_chain_selector(p, coin))
            .collect();
        if legacy.is_empty() {
            return None;
        }
        legacy.sort_by(|a, b| {
            let a_legacy = a
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.contains("legacy"))
                .unwrap_or(false);
            let b_legacy = b
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.contains("legacy"))
                .unwrap_or(false);
            b_legacy
                .cmp(&a_legacy)
                .then_with(|| rank_sidecar_candidate(b, coin).cmp(&rank_sidecar_candidate(a, coin)))
        });
        return legacy.into_iter().next();
    }
    let staged = staged_sidecar_path(coin);
    if staged.is_file() && is_real_sidecar(&staged) {
        if coin != CoinId::Verium || !binary_supports_unified_chain_selector(&staged, coin) {
            return Some(staged);
        }
    }
    if let Some(path) = candidates
        .iter()
        .find(|p| binary_supports_unified_chain_selector(p, coin))
    {
        return Some(path.clone());
    }
    candidates.into_iter().next()
}

fn detect_sidecar_binary(coin: CoinId) -> Option<PathBuf> {
    let name = coin.binary_name();
    let base = coin.binary_base();
    let exe = std::env::current_exe().ok()?;
    let parent = exe.parent()?;
    let mut candidates: Vec<PathBuf> = Vec::new();

    candidates.push(parent.join(&name));
    candidates.push(staged_sidecar_path(coin));

    if let Ok(entries) = std::fs::read_dir(parent) {
        for entry in entries.flatten() {
            let p = entry.path();
            let fname = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let matches_ext = if cfg!(target_os = "windows") {
                fname.ends_with(".exe")
            } else {
                !fname.contains('.') || fname.ends_with(".bin")
            };
            if fname.starts_with(&format!("{base}-")) && matches_ext && p.is_file() {
                candidates.push(p);
            }
        }
    }

    if let Ok(rustc_triple) = std::env::var("TARGET") {
        let triple_name = if cfg!(target_os = "windows") {
            format!("{base}-{rustc_triple}.exe")
        } else {
            format!("{base}-{rustc_triple}")
        };
        candidates.push(parent.join(&triple_name));
    }

    for dir in [
        parent.join("binaries"),
        parent.join("..").join("..").join("binaries"),
        parent.join("..").join("..").join("..").join("binaries"),
    ] {
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let fname = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                let matches_ext = if cfg!(target_os = "windows") {
                    fname.ends_with(".exe")
                } else {
                    !fname.contains('.') || fname.ends_with(".bin")
                };
                if fname.starts_with(&format!("{base}-")) && matches_ext && p.is_file() {
                    candidates.push(p);
                }
            }
        }
    }

    pick_preferred_sidecar(coin, candidates)
}

fn detect_unified_veriumd_for_vericoin() -> Option<PathBuf> {
    let path = detect_sidecar_binary(CoinId::Verium)?;
    if binary_supports_unified_chain_selector(&path, CoinId::Vericoin) {
        Some(path)
    } else {
        None
    }
}

fn legacy_sidecar_acceptable(coin: CoinId, path: &Path) -> bool {
    if coin != CoinId::Verium {
        return true;
    }
    !binary_supports_unified_chain_selector(path, coin)
}

fn detect_native_binary(coin: CoinId) -> DaemonBinaryStatus {
    let name = coin.binary_name();
    let env_var = match coin {
        CoinId::Verium => "VERIUMD_PATH",
        CoinId::Vericoin => "VERICOIND_PATH",
    };

    if let Ok(p) = std::env::var(env_var) {
        let path = PathBuf::from(p);
        if is_real_sidecar(&path) && legacy_sidecar_acceptable(coin, &path) {
            return DaemonBinaryStatus {
                found: true,
                path: Some(path.display().to_string()),
                source: DaemonBinarySource::Env,
                manageable: true,
                runtime: "native".into(),
                coin: coin.as_str().to_string(),
                stub_sidecar: false,
                missing_hint: None,
            };
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join(&name);
            if is_real_sidecar(&candidate) && legacy_sidecar_acceptable(coin, &candidate) {
                return DaemonBinaryStatus {
                    found: true,
                    path: Some(candidate.display().to_string()),
                    source: DaemonBinarySource::AdjacentToApp,
                    manageable: true,
                    runtime: "native".into(),
                    coin: coin.as_str().to_string(),
                    stub_sidecar: false,
                    missing_hint: None,
                };
            }
        }
    }

    if let Ok(p) = which::which(&name) {
        if is_real_sidecar(&p) && legacy_sidecar_acceptable(coin, &p) {
            return DaemonBinaryStatus {
                found: true,
                path: Some(p.display().to_string()),
                source: DaemonBinarySource::Path,
                manageable: true,
                runtime: "native".into(),
                coin: coin.as_str().to_string(),
                stub_sidecar: false,
                missing_hint: None,
            };
        }
    }

    unavailable_binary_status(coin)
}

fn sidecar_stub_path(coin: CoinId) -> Option<PathBuf> {
    let base = coin.binary_base();
    let exe = std::env::current_exe().ok()?;
    let parent = exe.parent()?;
    let mut candidates: Vec<PathBuf> = vec![parent.join(coin.binary_name())];
    for dir in [
        parent.join("binaries"),
        parent.join("..").join("..").join("binaries"),
        parent.join("..").join("..").join("..").join("binaries"),
    ] {
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let fname = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                let matches_ext = if cfg!(target_os = "windows") {
                    fname.ends_with(".exe")
                } else {
                    !fname.contains('.') || fname.ends_with(".bin")
                };
                if fname.starts_with(&format!("{base}-")) && matches_ext && p.is_file() {
                    candidates.push(p);
                }
            }
        }
    }
    candidates
        .into_iter()
        .find(|p| p.is_file() && !is_real_sidecar(p))
}

fn sidecar_stub_present(coin: CoinId) -> bool {
    sidecar_stub_path(coin).is_some()
}

pub fn binary_missing_hint(coin: CoinId) -> Option<String> {
    if detect_binary(coin).found {
        return None;
    }
    let name = coin.binary_base();
    if coin == CoinId::Verium {
        return Some(format!(
            "{name} was not found. Verium mainnet requires the legacy flat-layout {name} \
             (verium-only v1.x — not the unified vericoin/veriumd build). Set VERIUMD_PATH \
             to a verium-only binary or build from verium/ with --without-gui."
        ));
    }
    if sidecar_stub_present(coin) {
        Some(format!(
            "The bundled {name} is a build placeholder only — install a real {name} binary \
             (set {name}_PATH, or copy a real binary to desktop-app/run/{name})."
        ))
    } else {
        Some(format!(
            "{name} was not found on this system. Install the {name} node binary or set \
             {}_PATH to an existing build.",
            name.to_uppercase()
        ))
    }
}

fn unavailable_binary_status(coin: CoinId) -> DaemonBinaryStatus {
    let stub_sidecar = sidecar_stub_present(coin);
    DaemonBinaryStatus {
        found: false,
        path: sidecar_stub_path(coin).map(|p| p.display().to_string()),
        source: DaemonBinarySource::None,
        manageable: false,
        runtime: "none".into(),
        coin: coin.as_str().to_string(),
        stub_sidecar,
        missing_hint: binary_missing_hint(coin),
    }
}

pub fn detect_binary(coin: CoinId) -> DaemonBinaryStatus {
    if let Some(sidecar) = detect_sidecar_binary(coin) {
        return DaemonBinaryStatus {
            found: true,
            path: Some(sidecar.display().to_string()),
            source: DaemonBinarySource::Sidecar,
            manageable: true,
            runtime: "bundled".into(),
            coin: coin.as_str().to_string(),
            stub_sidecar: false,
            missing_hint: None,
        };
    }

    if coin == CoinId::Vericoin {
        if let Some(path) = detect_unified_veriumd_for_vericoin() {
            return DaemonBinaryStatus {
                found: true,
                path: Some(path.display().to_string()),
                source: DaemonBinarySource::Sidecar,
                manageable: true,
                runtime: "unified_veriumd".into(),
                coin: coin.as_str().to_string(),
                stub_sidecar: false,
                missing_hint: None,
            };
        }
    }

    let native = detect_native_binary(coin);
    if native.found {
        return native;
    }

    unavailable_binary_status(coin)
}

pub fn resolve_daemon_binary(coin: CoinId) -> Option<PathBuf> {
    detect_binary(coin).path.map(PathBuf::from)
}

fn cleanup_alternate_staged_sidecars(coin: CoinId, run_dir: &Path, keep: &Path) {
    let prefix = format!("{}-", coin.binary_base());
    let Ok(entries) = std::fs::read_dir(run_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path == keep {
            continue;
        }
        let Some(fname) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        let is_alt = fname.starts_with(&prefix) && (fname.ends_with(".exe") || !fname.contains('.'));
        if is_alt && path.is_file() {
            let _ = std::fs::remove_file(&path);
        }
    }
}

fn copy_staged_sidecar(coin: CoinId, source: &Path, dest: &Path) -> HostResult<PathBuf> {
    match std::fs::copy(source, dest) {
        Ok(_) => Ok(dest.to_path_buf()),
        Err(e) if e.raw_os_error() == Some(32) && dest.is_file() => Ok(dest.to_path_buf()),
        Err(e) => Err(HostError::other(format!(
            "could not stage {} for spawn (stop {} and retry): {e}",
            source.display(),
            coin.binary_base()
        ))),
    }
}

pub fn stage_sidecar_for_spawn(coin: CoinId, source: &Path) -> HostResult<PathBuf> {
    if !is_daemon_sidecar(source) {
        return Ok(source.to_path_buf());
    }
    let run_dir = app_config_base().join("run");
    std::fs::create_dir_all(&run_dir)?;
    let dest = run_dir.join(coin.binary_name());
    if source == dest.as_path() {
        cleanup_alternate_staged_sidecars(coin, &run_dir, &dest);
        return Ok(dest);
    }
    let staged = if dest.is_file() {
        if let (Ok(src_meta), Ok(dst_meta)) = (source.metadata(), dest.metadata()) {
            let same_bytes = src_meta.len() == dst_meta.len();
            let dest_not_older = dst_meta.modified().ok() >= src_meta.modified().ok();
            if same_bytes && dest_not_older {
                dest.clone()
            } else {
                copy_staged_sidecar(coin, source, &dest)?
            }
        } else {
            copy_staged_sidecar(coin, source, &dest)?
        }
    } else {
        copy_staged_sidecar(coin, source, &dest)?
    };
    cleanup_alternate_staged_sidecars(coin, &run_dir, &staged);
    Ok(staged)
}
