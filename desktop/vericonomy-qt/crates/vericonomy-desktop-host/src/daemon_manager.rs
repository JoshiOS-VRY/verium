//! Managed daemon subprocess lifecycle — port of Tauri `DaemonManager`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use crate::coin::CoinId;
use crate::context::AppContext;
use crate::daemon_binary::{
    binary_missing_hint, binary_supports_native_pool_mining, binary_supports_unified_chain_selector,
    is_real_sidecar, resolve_daemon_binary, stage_sidecar_for_spawn,
};
use crate::daemon_config::{self, sync_rpc_from_conf, verium_uses_legacy_flat, DaemonConfig};
use crate::error::{HostError, HostResult};
use crate::rpc::{RpcClient, RpcEndpoint};

const DAEMON_UACOMMENT: &str = "alpha1";

static MANAGERS: std::sync::OnceLock<Mutex<HashMap<CoinId, Arc<DaemonManager>>>> =
    std::sync::OnceLock::new();

fn managers() -> &'static Mutex<HashMap<CoinId, Arc<DaemonManager>>> {
    MANAGERS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub async fn manager(coin: CoinId) -> Arc<DaemonManager> {
    let mut guard = managers().lock().await;
    if let Some(m) = guard.get(&coin) {
        return m.clone();
    }
    let m = Arc::new(DaemonManager::new(coin));
    guard.insert(coin, m.clone());
    m
}

pub fn daemon_runtime_overrides(legacy_flat: bool) -> Vec<(&'static str, String)> {
    let mut overrides = vec![
        ("dbcache", "450".to_string()),
        ("par", "4".to_string()),
        ("maxconnections", "16".to_string()),
        ("maxuploadtarget", "0".to_string()),
        ("rpcworkqueue", "256".to_string()),
        ("rpcthreads", "16".to_string()),
    ];
    if !legacy_flat {
        overrides.push(("maxmempool", "50".to_string()));
    }
    overrides
}

pub struct DaemonManager {
    coin: CoinId,
    child: Arc<Mutex<Option<Child>>>,
    managed: Arc<Mutex<bool>>,
    spawn_binary: Arc<Mutex<Option<PathBuf>>>,
}

impl DaemonManager {
    pub fn new(coin: CoinId) -> Self {
        Self {
            coin,
            child: Arc::new(Mutex::new(None)),
            managed: Arc::new(Mutex::new(false)),
            spawn_binary: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn start(&self, cfg: &DaemonConfig, extra_args: &[&str]) -> HostResult<u32> {
        {
            let mut child = self.child.lock().await;
            if let Some(ref mut process) = *child {
                match process.try_wait() {
                    Ok(None) => return Ok(0),
                    Ok(Some(_)) | Err(_) => {
                        *child = None;
                        *self.managed.lock().await = false;
                    }
                }
            }
        }

        let bin = {
            let mut cache = self.spawn_binary.lock().await;
            if let Some(path) = cache.as_ref() {
                if path.is_file() && is_real_sidecar(path) {
                    path.clone()
                } else {
                    cache.take();
                    self.resolve_and_stage_binary(cfg).await?
                }
            } else {
                let path = self.resolve_and_stage_binary(cfg).await?;
                *cache = Some(path.clone());
                path
            }
        };

        let legacy_flat = self.coin == CoinId::Verium && verium_uses_legacy_flat(cfg);
        if legacy_flat && binary_supports_unified_chain_selector(&bin, self.coin) {
            if !binary_supports_native_pool_mining(&bin) {
                return Err(HostError::other(format!(
                    "Refusing to start unified vericoin/veriumd for Verium mainnet ({}) — \
                     install the legacy verium-only v1.x sidecar.",
                    bin.display()
                )));
            }
        }
        let unified_chain =
            !legacy_flat && binary_supports_unified_chain_selector(&bin, self.coin);

        let mut spawn_cfg = cfg.clone();
        sync_rpc_from_conf(self.coin, &mut spawn_cfg)?;

        let mut std_cmd = std::process::Command::new(&bin);
        std_cmd
            .arg(format!("-datadir={}", spawn_cfg.datadir.display()))
            .arg("-server=1")
            .arg("-checklevel=0")
            .arg(format!("-rpcport={}", spawn_cfg.rpc_port))
            .arg(format!("-rpcbind={}", spawn_cfg.rpc_host))
            .arg("-rpcallowip=127.0.0.1")
            .arg(format!("-uacomment={DAEMON_UACOMMENT}"))
            .arg("-printtoconsole=0");
        if legacy_flat {
            std_cmd.arg("-conf=verium.conf");
        }
        for (key, value) in daemon_runtime_overrides(legacy_flat) {
            std_cmd.arg(format!("-{key}={value}"));
        }
        if let Some(user) = spawn_cfg.rpc_user.as_deref().filter(|u| !u.is_empty()) {
            std_cmd.arg(format!("-rpcuser={user}"));
        }
        if let Some(pass) = spawn_cfg.rpc_password.as_deref().filter(|p| !p.is_empty()) {
            std_cmd.arg(format!("-rpcpassword={pass}"));
        }

        match self.coin {
            CoinId::Vericoin => {
                if binary_supports_unified_chain_selector(&bin, CoinId::Verium)
                    && !binary_supports_unified_chain_selector(&bin, CoinId::Vericoin)
                {
                    return Err(HostError::other(format!(
                        "Refusing to start verium-only {} for Vericoin",
                        bin.display()
                    )));
                }
                std_cmd.arg("-vericoin");
            }
            CoinId::Verium if unified_chain => {
                std_cmd.arg("-verium");
            }
            _ if unified_chain => {
                if let Some(chain_arg) = self.coin.chain_cli_arg() {
                    std_cmd.arg(chain_arg);
                }
            }
            _ => {}
        }
        if spawn_cfg.chain == "test" {
            std_cmd.arg("-testnet");
        }
        for arg in extra_args {
            if !arg.is_empty() {
                std_cmd.arg(*arg);
            }
        }
        std_cmd.current_dir(&spawn_cfg.datadir);

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            std_cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let mut cmd = Command::from(std_cmd);
        cmd.kill_on_drop(false);
        let child = cmd.spawn().map_err(|e| {
            HostError::other(format!(
                "failed to spawn {}: {e}",
                self.coin.binary_base()
            ))
        })?;
        let pid = child.id().unwrap_or(0);
        *self.child.lock().await = Some(child);
        *self.managed.lock().await = true;
        Ok(pid)
    }

    pub async fn mark_managed(&self) {
        *self.managed.lock().await = true;
    }

    pub async fn is_managed(&self) -> bool {
        *self.managed.lock().await
    }

    pub async fn clear_tracking(&self) {
        *self.managed.lock().await = false;
        *self.child.lock().await = None;
    }

    async fn resolve_and_stage_binary(&self, _cfg: &DaemonConfig) -> HostResult<PathBuf> {
        let bin = resolve_daemon_binary(self.coin).ok_or_else(|| {
            HostError::other(
                binary_missing_hint(self.coin).unwrap_or_else(|| {
                    format!("could not locate {} binary", self.coin.binary_base())
                }),
            )
        })?;
        if !is_real_sidecar(&bin) {
            return Err(HostError::other(
                binary_missing_hint(self.coin).unwrap_or_else(|| {
                    format!(
                        "refusing to run build placeholder at {}",
                        bin.display()
                    )
                }),
            ));
        }
        stage_sidecar_for_spawn(self.coin, &bin)
    }

    pub async fn wait_for_child_exit(&self, timeout: Duration) {
        let mut child = self.child.lock().await;
        if let Some(ref mut process) = *child {
            tokio::select! {
                _ = process.wait() => {}
                _ = tokio::time::sleep(timeout) => {}
            }
        }
    }

    pub async fn force_kill_child(&self) {
        if let Some(mut process) = self.child.lock().await.take() {
            let _ = process.start_kill();
            let _ = process.wait().await;
        }
    }
}

#[cfg(windows)]
pub fn pids_listening_on_port(port: u16) -> Vec<u32> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let needle = format!(":{port}");
    let output = match std::process::Command::new("netstat")
        .args(["-ano"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
    {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };
    if !output.status.success() {
        return Vec::new();
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut pids = Vec::new();
    for line in text.lines() {
        if !line.contains("LISTENING") || !line.contains(&needle) {
            continue;
        }
        let Some(pid) = line.split_whitespace().last().and_then(|s| s.parse().ok()) else {
            continue;
        };
        if pid > 0 {
            pids.push(pid);
        }
    }
    pids.sort_unstable();
    pids.dedup();
    pids
}

#[cfg(not(windows))]
pub fn pids_listening_on_port(port: u16) -> Vec<u32> {
    let output = std::process::Command::new("lsof")
        .args(["-nP", &format!("-iTCP:{port}"), "-sTCP:LISTEN", "-t"])
        .output()
        .ok();
    let Some(output) = output else {
        return Vec::new();
    };
    let mut pids: Vec<u32> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .filter(|p| *p > 0)
        .collect();
    pids.sort_unstable();
    pids.dedup();
    pids
}

pub fn kill_port_listeners(port: u16) {
    for pid in pids_listening_on_port(port) {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            use std::process::Stdio;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            let _ = std::process::Command::new("taskkill")
                .args(["/F", "/PID", &pid.to_string()])
                .creation_flags(CREATE_NO_WINDOW)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        #[cfg(not(windows))]
        {
            use std::process::Stdio;
            let _ = std::process::Command::new("kill")
                .args(["-9", &pid.to_string()])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }
}

#[cfg(windows)]
pub fn force_stop_native_daemon(coin: CoinId) {
    use std::os::windows::process::CommandExt;
    use std::process::Stdio;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let prefix_lower = coin.binary_base().to_ascii_lowercase();
    let output = match std::process::Command::new("tasklist")
        .args(["/NH", "/FO", "TABLE"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
    {
        Ok(o) => o,
        Err(_) => return,
    };
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let line = line.trim();
        let mut parts = line.split_whitespace();
        let Some(image) = parts.next() else { continue };
        if !image.to_ascii_lowercase().starts_with(&prefix_lower) {
            continue;
        }
        let Some(pid_str) = parts.next() else { continue };
        if let Ok(pid) = pid_str.parse::<u32>() {
            let _ = std::process::Command::new("taskkill")
                .args(["/F", "/PID", &pid.to_string()])
                .creation_flags(CREATE_NO_WINDOW)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }
}

#[cfg(not(windows))]
pub fn force_stop_native_daemon(coin: CoinId) {
    let output = match std::process::Command::new("ps")
        .args(["-A", "-o", "pid=,comm="])
        .output()
    {
        Ok(o) => o,
        Err(_) => return,
    };
    let prefix = coin.binary_base();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let line = line.trim();
        let Some((pid_str, comm)) = line.split_once(char::is_whitespace) else {
            continue;
        };
        let name = comm.trim().rsplit(['/', '\\']).next().unwrap_or(comm.trim());
        if !name.starts_with(prefix) {
            continue;
        }
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            use std::process::Stdio;
            let _ = std::process::Command::new("kill")
                .args(["-9", &pid.to_string()])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }
}

pub fn free_rpc_port(coin: CoinId, cfg: &DaemonConfig) {
    kill_port_listeners(cfg.rpc_port);
    force_stop_native_daemon(coin);
}

pub fn native_daemon_image_running(coin: CoinId) -> bool {
    #[cfg(windows)]
    {
        let prefix = coin.binary_base().to_ascii_lowercase();
        let output = match std::process::Command::new("tasklist")
            .args(["/NH", "/FO", "TABLE"])
            .output()
        {
            Ok(o) => o,
            Err(_) => return false,
        };
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line.to_ascii_lowercase().contains(&prefix))
    }
    #[cfg(not(windows))]
    {
        let output = match std::process::Command::new("ps")
            .args(["-A", "-o", "comm="])
            .output()
        {
            Ok(o) => o,
            Err(_) => return false,
        };
        let prefix = coin.binary_base();
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line.trim().starts_with(prefix))
    }
}

pub async fn rpc_reachable(ctx: &AppContext, _coin: CoinId, cfg: &DaemonConfig) -> bool {
    let user = cfg.rpc_user.as_deref().unwrap_or("");
    let pass = cfg.rpc_password.as_deref().unwrap_or("");
    if user.is_empty() || pass.is_empty() {
        return false;
    }
    let ep = RpcEndpoint::loopback(cfg.rpc_port, user, pass);
    let client = RpcClient::new(ctx.http(), &ep);
    client
        .call("getblockchaininfo", serde_json::json!([]))
        .await
        .is_ok()
}

pub fn sync_ctx_endpoint(ctx: &AppContext, coin: CoinId, cfg: &DaemonConfig) {
    if let (Some(user), Some(pass)) = (cfg.rpc_user.clone(), cfg.rpc_password.clone()) {
        if !user.is_empty() && !pass.is_empty() {
            ctx.set_endpoint(coin, RpcEndpoint::loopback(cfg.rpc_port, user, pass));
        }
    } else if let Some(ep) = crate::config::resolve_endpoint(coin) {
        ctx.set_endpoint(coin, ep);
    }
}

pub async fn start_managed_daemon(ctx: &AppContext, coin: CoinId) -> HostResult<DaemonConfig> {
    let mut cfg = daemon_config::load_daemon_config(coin)?;
    sync_rpc_from_conf(coin, &mut cfg)?;
    sync_ctx_endpoint(ctx, coin, &cfg);

    if rpc_reachable(ctx, coin, &cfg).await {
        manager(coin).await.mark_managed().await;
        return Ok(cfg);
    }

    if !pids_listening_on_port(cfg.rpc_port).is_empty() {
        if !rpc_reachable(ctx, coin, &cfg).await {
            kill_port_listeners(cfg.rpc_port);
            tokio::time::sleep(Duration::from_millis(1500)).await;
        } else {
            manager(coin).await.mark_managed().await;
            return Ok(cfg);
        }
    }

    if native_daemon_image_running(coin) && !rpc_reachable(ctx, coin, &cfg).await {
        free_rpc_port(coin, &cfg);
        tokio::time::sleep(Duration::from_millis(1500)).await;
    }

    let dm = manager(coin).await;
    dm.start(&cfg, &[]).await?;
    sync_ctx_endpoint(ctx, coin, &cfg);
    Ok(cfg)
}

pub async fn stop_managed_daemon(ctx: &AppContext, coin: CoinId) -> HostResult<()> {
    let mut cfg = daemon_config::load_daemon_config(coin)?;
    sync_rpc_from_conf(coin, &mut cfg)?;

    if coin == CoinId::Verium {
        crate::mining_supervisor::stop().await;
        if let Some(ep) = ctx.endpoint(coin) {
            let client = RpcClient::new(ctx.http(), &ep);
            let _ = client.call("minerstop", serde_json::json!([])).await;
        }
    } else if let Some(ep) = ctx.endpoint(coin) {
        let client = RpcClient::new(ctx.http(), &ep);
        let _ = client.call("stakingstop", serde_json::json!([])).await;
    }

    if let Some(ep) = ctx.endpoint(coin) {
        let client = RpcClient::new(ctx.http(), &ep);
        let _ = client.call("stop", serde_json::json!([])).await;
        manager(coin)
            .await
            .wait_for_child_exit(Duration::from_secs(30))
            .await;
    }

    manager(coin).await.force_kill_child().await;
    if !pids_listening_on_port(cfg.rpc_port).is_empty() {
        free_rpc_port(coin, &cfg);
    }
    manager(coin).await.clear_tracking().await;
    Ok(())
}

pub async fn restart_managed_daemon(ctx: &AppContext, coin: CoinId) -> HostResult<DaemonConfig> {
    let _ = stop_managed_daemon(ctx, coin).await;
    tokio::time::sleep(Duration::from_millis(1000)).await;
    start_managed_daemon(ctx, coin).await
}
