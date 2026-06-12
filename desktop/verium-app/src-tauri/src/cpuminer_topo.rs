//! CPU topology helpers matching veriumMiner `topo.c` / `topo_recommended_threads`.
//!
//! Pool mining prefers the bundled cpuminer's own `--tune` / `Auto threads` output
//! so the wallet matches what `veriumMiner -t 0` picks on this machine. The Rust
//! port below is only a fallback when the sidecar binary is missing or tune fails.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use sysinfo::System;

const SCRYPT_N: u64 = 1_048_576;

fn os_reserve_bytes(total_ram_bytes: u64) -> u64 {
    if total_ram_bytes == 0 {
        return 2 * 1024 * 1024 * 1024;
    }
    let mut r = total_ram_bytes / 8;
    r = r.max(1024 * 1024 * 1024);
    r = r.min(4 * 1024 * 1024 * 1024);
    r
}

#[cfg(target_os = "linux")]
fn probe_p_logical_linux() -> u32 {
    let logical = std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(1);
    let mut p = 0u32;
    for cpu in 0..logical {
        let path = format!("/sys/devices/system/cpu/cpu{cpu}/topology/core_type");
        let Ok(raw) = std::fs::read_to_string(&path) else {
            continue;
        };
        if raw.trim() == "2" {
            p += 1;
        }
    }
    p
}

#[cfg(not(target_os = "linux"))]
fn probe_p_logical_linux() -> u32 {
    0
}

fn arm_bandwidth_limited_sbc(topo: &TopoSnapshot, scratchpad_bytes: u64) -> bool {
    #[cfg(target_arch = "aarch64")]
    {
        if topo.logical_cpus != topo.physical_cpus || topo.physical_cpus > 8 {
            return false;
        }
        if scratchpad_bytes <= 200 * 1024 * 1024 {
            return false;
        }
        if topo.l3_bytes >= 32 * 1024 * 1024 {
            return false;
        }
        return true;
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        let _ = (topo, scratchpad_bytes);
        false
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TopoSnapshot {
    pub logical_cpus: u32,
    pub physical_cpus: u32,
    pub performance_cpus: u32,
    pub l3_bytes: u64,
    pub total_ram_bytes: u64,
    pub avail_ram_bytes: u64,
}

/// Match cpuminer `scrypt_scratchpad_bytes(N)` with AVX2 multi-lane ROM (6 lanes).
pub fn cpuminer_scratchpad_bytes(avx2: bool) -> u64 {
    let lanes = if avx2 { 6 } else { 1 };
    SCRYPT_N * lanes * 128 + 63
}

pub fn cpuminer_scratchpad_mib(avx2: bool) -> u32 {
    ((cpuminer_scratchpad_bytes(avx2) + 1024 * 1024 - 1) / (1024 * 1024)) as u32
}

fn detect_x86_avx2() -> bool {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        std::arch::is_x86_feature_detected!("avx2")
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        false
    }
}

fn probe_l3_bytes_linux() -> u64 {
    #[cfg(target_os = "linux")]
    {
        let mut max_l3 = 0u64;
        for cpu in 0..4 {
            for idx in 0..8 {
                let path = format!(
                    "/sys/devices/system/cpu/cpu{cpu}/cache/index{idx}/size"
                );
                let Ok(raw) = std::fs::read_to_string(&path) else {
                    continue;
                };
                if let Some(bytes) = parse_cache_size(&raw) {
                    max_l3 = max_l3.max(bytes);
                }
            }
        }
        return max_l3;
    }
    #[cfg(not(target_os = "linux"))]
    {
        0
    }
}

#[cfg(target_os = "linux")]
fn parse_cache_size(raw: &str) -> Option<u64> {
    let s = raw.trim();
    let (num, suffix) = s.split_at(s.len().saturating_sub(1));
    let value: u64 = num.trim().parse().ok()?;
    Some(match suffix.chars().last()? {
        'K' | 'k' => value * 1024,
        'M' | 'm' => value * 1024 * 1024,
        'G' | 'g' => value * 1024 * 1024 * 1024,
        _ => value,
    })
}

fn hybrid_performance_cpus(logical: u32, _physical: u32) -> u32 {
    let p = probe_p_logical_linux();
    if p > 0 && p < logical {
        p
    } else {
        0
    }
}

pub fn probe_topo() -> TopoSnapshot {
    let logical = std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(1)
        .max(1);
    let sys = System::new();
    let physical = sys
        .physical_core_count()
        .map(|n| n as u32)
        .filter(|&n| n > 0)
        .unwrap_or(logical)
        .max(1);

    let performance_cpus = hybrid_performance_cpus(logical, physical);

    let mut sys = sys;
    sys.refresh_memory();
    let total_ram_bytes = sys.total_memory();
    let avail_ram_bytes = sys.available_memory();

    TopoSnapshot {
        logical_cpus: logical,
        physical_cpus: physical,
        performance_cpus,
        l3_bytes: probe_l3_bytes_linux(),
        total_ram_bytes,
        avail_ram_bytes,
    }
}

/// Port of veriumMiner `topo_recommended_threads(scratchpad_bytes)`.
pub fn recommended_threads(scratchpad_bytes: u64, topo: &TopoSnapshot) -> u32 {
    let exec = topo.logical_cpus.max(1);
    let reserve = os_reserve_bytes(topo.total_ram_bytes);

    let mut ram_for_miner = topo.avail_ram_bytes;
    if ram_for_miner < reserve && topo.total_ram_bytes > reserve {
        ram_for_miner = topo.total_ram_bytes - reserve;
    } else if ram_for_miner > reserve {
        ram_for_miner -= reserve;
    } else if topo.total_ram_bytes > reserve {
        ram_for_miner = topo.total_ram_bytes - reserve;
    } else {
        ram_for_miner = 0;
    }

    let mut by_ram = exec;
    if scratchpad_bytes > 0 && ram_for_miner > 0 {
        by_ram = (ram_for_miner / scratchpad_bytes).max(1) as u32;
    }

    let mut rec = exec.min(by_ram);
    if arm_bandwidth_limited_sbc(topo, scratchpad_bytes) && rec > 1 {
        rec = 1;
    }
    rec.max(1)
}

struct TuneCacheEntry {
    binary: PathBuf,
    threads: u32,
    at: Instant,
}

static TUNE_CACHE: Mutex<Option<TuneCacheEntry>> = Mutex::new(None);
const TUNE_CACHE_TTL: Duration = Duration::from_secs(300);
const TUNE_PROBE_WAIT: Duration = Duration::from_millis(2500);

/// Parse cpuminer startup lines: `Auto threads: N` or `recommended=N threads`.
fn parse_tune_output(text: &str) -> Option<u32> {
    for line in text.lines() {
        if let Some(rest) = line.split("Auto threads:").nth(1) {
            if let Ok(n) = rest.trim().split_whitespace().next()?.parse::<u32>() {
                if n > 0 {
                    return Some(n);
                }
            }
        }
        if let Some(idx) = line.find("recommended=") {
            let rest = &line[idx + "recommended=".len()..];
            let token = rest.split_whitespace().next()?;
            let num = token.trim_end_matches(" threads").trim_end_matches(',');
            if let Ok(n) = num.parse::<u32>() {
                if n > 0 {
                    return Some(n);
                }
            }
        }
    }
    None
}

/// Ask the bundled cpuminer what `-t 0` would select (`--benchmark --tune`).
pub fn query_cpuminer_auto_threads(binary: &Path) -> Option<u32> {
    if !binary.is_file() {
        return None;
    }
    if let Ok(cache) = TUNE_CACHE.lock() {
        if let Some(entry) = cache.as_ref() {
            if entry.binary == binary && entry.at.elapsed() < TUNE_CACHE_TTL {
                return Some(entry.threads);
            }
        }
    }

    let mut cmd = Command::new(binary);
    cmd.args(["--benchmark", "--tune", "-t", "0", "--no-color"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = cmd.spawn().ok()?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let reader = std::thread::spawn(move || {
        let mut text = String::new();
        if let Some(mut out) = stdout {
            let _ = out.read_to_string(&mut text);
        }
        if let Some(mut err) = stderr {
            let _ = err.read_to_string(&mut text);
        }
        text
    });

    std::thread::sleep(TUNE_PROBE_WAIT);
    let _ = child.kill();
    let _ = child.wait();
    let text = reader.join().ok()?;
    let threads = parse_tune_output(&text)?;

    if let Ok(mut cache) = TUNE_CACHE.lock() {
        *cache = Some(TuneCacheEntry {
            binary: binary.to_path_buf(),
            threads,
            at: Instant::now(),
        });
    }
    Some(threads)
}

/// Estimate from topology when the sidecar cannot be queried.
pub fn cpuminer_recommended_threads_estimated() -> u32 {
    let avx2 = detect_x86_avx2();
    let topo = probe_topo();
    recommended_threads(cpuminer_scratchpad_bytes(avx2), &topo)
}

/// Prefer the sidecar binary's own tune output; fall back to the Rust port.
pub fn cpuminer_recommended_threads(binary: Option<&Path>) -> u32 {
    if let Some(path) = binary {
        if let Some(n) = query_cpuminer_auto_threads(path) {
            return n;
        }
    }
    cpuminer_recommended_threads_estimated()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scratchpad_matches_cpuminer_avx2() {
        // 1048576 * 6 * 128 + 63
        assert_eq!(cpuminer_scratchpad_bytes(true), 805_306_431);
    }

    #[test]
    fn recommends_logical_cpus_when_ram_allows() {
        let topo = TopoSnapshot {
            logical_cpus: 24,
            physical_cpus: 16,
            performance_cpus: 16,
            l3_bytes: 0,
            total_ram_bytes: 64 * 1024 * 1024 * 1024,
            avail_ram_bytes: 48 * 1024 * 1024 * 1024,
        };
        let sp = cpuminer_scratchpad_bytes(true);
        assert_eq!(recommended_threads(sp, &topo), 24);
    }

    #[test]
    fn hybrid_intel_uses_logical_when_ram_allows() {
        let topo = TopoSnapshot {
            logical_cpus: 24,
            physical_cpus: 16,
            performance_cpus: 16,
            l3_bytes: 0,
            total_ram_bytes: 64 * 1024 * 1024 * 1024,
            avail_ram_bytes: 48 * 1024 * 1024 * 1024,
        };
        let sp = cpuminer_scratchpad_bytes(true);
        assert_eq!(recommended_threads(sp, &topo), 24);
    }
}
