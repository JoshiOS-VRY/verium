//! Resolve RPC endpoints from the on-disk `vericonomy.conf`.
//!
//! Minimal port of the parts of `src-tauri/src/config.rs` needed to connect:
//! locate the platform data dir, read the `[verium]` / `[vericoin]` section, and
//! build an [`RpcEndpoint`]. Managed daemons use static `rpcuser`/`rpcpassword`
//! (no cookie), per `docs/SECURITY.md`.

use std::path::PathBuf;

use crate::coin::CoinId;
use crate::rpc::RpcEndpoint;

/// Platform application-data directory holding `vericonomy.conf` and chain data.
///
/// - Windows: `%APPDATA%\Verium`
/// - macOS:   `~/Library/Application Support/Verium`
/// - Linux:   `~/.verium`
pub fn node_conf_dir() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        dirs::home_dir().map(|h| h.join(".verium"))
    }
    #[cfg(not(target_os = "linux"))]
    {
        dirs::data_dir().map(|d| d.join("Verium"))
    }
}

/// Wallet app config — **same path as Tauri** (`config::app_config_base`).
///
/// macOS: `~/Library/Application Support/Vericonomy/desktop-app`
pub fn app_config_base() -> PathBuf {
    let base = dirs::config_dir()
        .or_else(dirs::data_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("Vericonomy").join("desktop-app")
}

fn section_name(coin: CoinId) -> &'static str {
    match coin {
        CoinId::Verium => "verium",
        CoinId::Vericoin => "vericoin",
    }
}

/// Resolve the RPC endpoint for `coin` by reading `vericonomy.conf`.
///
/// Returns `None` if the conf is missing or lacks credentials; callers fall back
/// to a port-only endpoint (which surfaces as "Offline" until configured).
pub fn resolve_endpoint(coin: CoinId) -> Option<RpcEndpoint> {
    let conf_path = node_conf_dir()?.join("vericonomy.conf");
    let contents = std::fs::read_to_string(conf_path).ok()?;
    parse_endpoint(&contents, coin)
}

/// Parse an [`RpcEndpoint`] for `coin` out of `vericonomy.conf` contents.
/// Separated from disk I/O so it can be unit-tested.
pub fn parse_endpoint(contents: &str, coin: CoinId) -> Option<RpcEndpoint> {
    let want = section_name(coin);
    let mut in_section = false;
    let mut user = None;
    let mut password = None;
    let mut port = None;

    for raw in contents.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            in_section = name.trim().eq_ignore_ascii_case(want);
            continue;
        }
        if !in_section {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            match k.trim() {
                "rpcuser" => user = Some(v.trim().to_string()),
                "rpcpassword" => password = Some(v.trim().to_string()),
                "rpcport" => port = v.trim().parse::<u16>().ok(),
                _ => {}
            }
        }
    }

    let port = port.unwrap_or_else(|| coin.default_rpc_port());
    match (user, password) {
        (Some(u), Some(p)) => Some(RpcEndpoint::loopback(port, u, p)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONF: &str = "\
# global
[verium]
rpcuser = veriumrpc
rpcpassword = s3cret
rpcport = 33987

[vericoin]
rpcuser = vrcrpc
rpcpassword = hunter2
";

    #[test]
    fn parses_verium_section() {
        let ep = parse_endpoint(CONF, CoinId::Verium).expect("verium endpoint");
        assert_eq!(ep.user, "veriumrpc");
        assert_eq!(ep.password, "s3cret");
        assert_eq!(ep.url, "http://127.0.0.1:33987");
    }

    #[test]
    fn falls_back_to_default_port() {
        let ep = parse_endpoint(CONF, CoinId::Vericoin).expect("vericoin endpoint");
        assert_eq!(ep.user, "vrcrpc");
        // No rpcport in the [vericoin] section -> default.
        assert_eq!(ep.url, "http://127.0.0.1:33988");
    }

    #[test]
    fn missing_credentials_yields_none() {
        assert!(parse_endpoint("[verium]\n", CoinId::Verium).is_none());
    }
}
