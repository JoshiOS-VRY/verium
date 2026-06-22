//! Daemon config + wallet.dat paths (Tauri-compatible `daemon-{coin}.json`).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::coin::CoinId;
use crate::error::{HostError, HostResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub datadir: PathBuf,
    pub rpc_host: String,
    pub rpc_port: u16,
    pub chain: String,
    #[serde(default)]
    pub rpc_user: Option<String>,
    #[serde(skip_serializing, default)]
    pub rpc_password: Option<String>,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        default_config(CoinId::Verium)
    }
}

pub fn default_config(coin: CoinId) -> DaemonConfig {
    let datadir = default_datadir(coin);
    DaemonConfig {
        datadir,
        rpc_host: "127.0.0.1".into(),
        rpc_port: coin.default_rpc_port(),
        chain: "main".into(),
        rpc_user: None,
        rpc_password: None,
    }
}

pub fn verium_uses_legacy_flat(cfg: &DaemonConfig) -> bool {
    cfg.chain == "main"
}

/// Pull RPC credentials from `vericonomy.conf` into the in-memory config.
pub fn sync_rpc_from_conf(coin: CoinId, cfg: &mut DaemonConfig) -> HostResult<()> {
    if let Some(ep) = crate::config::resolve_endpoint(coin) {
        cfg.rpc_user = Some(ep.user);
        cfg.rpc_password = Some(ep.password);
    }
    Ok(())
}

pub fn default_datadir(coin: CoinId) -> PathBuf {
    crate::config::node_conf_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(coin.data_dir_name())
}

pub fn app_daemon_config_path(coin: CoinId) -> PathBuf {
    crate::config::app_config_base().join(format!("daemon-{}.json", coin.as_str()))
}

#[derive(Debug, Deserialize)]
struct SavedDaemonConfig {
    datadir: PathBuf,
    rpc_host: String,
    rpc_port: u16,
    chain: String,
    #[serde(default)]
    rpc_user: Option<String>,
}

pub fn load_daemon_config(coin: CoinId) -> HostResult<DaemonConfig> {
    let path = app_daemon_config_path(coin);
    if path.exists() {
        let raw = std::fs::read_to_string(&path)?;
        if let Ok(saved) = serde_json::from_str::<SavedDaemonConfig>(&raw) {
            return Ok(DaemonConfig {
                datadir: saved.datadir,
                rpc_host: saved.rpc_host,
                rpc_port: saved.rpc_port,
                chain: saved.chain,
                rpc_user: saved.rpc_user,
                rpc_password: None,
            });
        }
    }
    Ok(default_config(coin))
}

pub fn chain_datadir(coin: CoinId, cfg: &DaemonConfig) -> PathBuf {
    cfg.datadir.join(coin.data_dir_name())
}

pub fn wallet_dat_path(coin: CoinId, cfg: &DaemonConfig) -> PathBuf {
    let base = chain_datadir(coin, cfg);
    for name in ["wallet.dat", "wallets/wallet.dat"] {
        let p = base.join(name);
        if p.exists() {
            return p;
        }
    }
    base.join("wallet.dat")
}

pub fn wallet_dat_exists(coin: CoinId, cfg: &DaemonConfig) -> bool {
    let path = wallet_dat_path(coin, cfg);
    path.is_file()
        && std::fs::metadata(&path)
            .map(|m| m.len() >= 512)
            .unwrap_or(false)
}

pub fn wallet_backup_dir(coin: CoinId, cfg: &DaemonConfig) -> HostResult<PathBuf> {
    let dir = chain_datadir(coin, cfg).join("backups");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn is_live_wallet_destination(coin: CoinId, cfg: &DaemonConfig, dest: &Path) -> bool {
    let live = wallet_dat_path(coin, cfg);
    dest.canonicalize()
        .ok()
        .zip(live.canonicalize().ok())
        .map(|(a, b)| a == b)
        .unwrap_or_else(|| {
            dest.to_string_lossy().eq_ignore_ascii_case(&live.to_string_lossy())
        })
}

pub fn clear_wallet_bdb_environment(wallet_path: &Path) -> HostResult<()> {
    let dir = wallet_path
        .parent()
        .ok_or_else(|| HostError::other("wallet path has no parent"))?;
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("database") || name.ends_with(".log") {
            let _ = std::fs::remove_file(entry.path());
        }
    }
    Ok(())
}
