use std::collections::HashMap;
use std::path::PathBuf;

use serde::Serialize;

pub use vericonomy_chain_params::{CoinId, CoinTarget, NetworkMode};

use crate::error::{AppError, AppResult};

// ---------------------------------------------------------------------------
// App-only CoinId helpers (datadir, explorer URLs, daemon RPC matching, etc.)
// Extension traits avoid orphan-rule issues with SDK re-exports.
// ---------------------------------------------------------------------------

pub trait CoinIdAppExt {
    fn binary_base(self) -> &'static str;
    fn conf_filename(self) -> &'static str;
    fn default_rpc_port(self) -> u16;
    fn default_p2p_port(self) -> u16;
    fn default_network_chain(self) -> &'static str;
    fn conf_section(self) -> Option<&'static str>;
    fn chain_cli_arg(self) -> Option<&'static str>;
    fn rpc_chain_matches(self, rpc_chain: &str, cfg_chain: &str) -> bool;
    fn default_datadir(self) -> PathBuf;
    fn bootstrap_cdn_base(self) -> &'static str;
    fn explorer_api_base(self) -> &'static str;
    fn explorer_chain_api_base(self) -> &'static str;
    fn explorer_indexer_chain_id(self) -> &'static str;
    fn explorer_logo_url(self) -> &'static str;
    fn confirmations_matured(self) -> u32;
    fn earn_mode(self) -> &'static str;
    fn keychain_service(self) -> String;
    fn default_rpc_user(self) -> &'static str;
    fn wallet_backup_prefix(self) -> &'static str;
    fn default_electrum_servers(self, network: NetworkMode) -> Vec<String>;
}

impl CoinIdAppExt for CoinId {
    fn binary_base(self) -> &'static str {
        match self {
            CoinId::Verium => "veriumd",
            CoinId::Vericoin => "vericoind",
        }
    }

    fn conf_filename(self) -> &'static str {
        // Unified vericoin/veriumd builds read vericonomy.conf (see BITCOIN_CONF_FILENAME).
        "vericonomy.conf"
    }

    fn default_rpc_port(self) -> u16 {
        self.profile().default_rpc_port
    }

    /// Default P2P listen port for outbound `addnode` targets on mainnet.
    fn default_p2p_port(self) -> u16 {
        self.profile().default_p2p_port
    }

    fn default_network_chain(self) -> &'static str {
        match self {
            CoinId::Verium => "main",
            CoinId::Vericoin => "vericoin",
        }
    }

    fn conf_section(self) -> Option<&'static str> {
        match self {
            CoinId::Verium => Some("verium"),
            CoinId::Vericoin => Some("vericoin"),
        }
    }

    fn chain_cli_arg(self) -> Option<&'static str> {
        match self {
            CoinId::Verium => Some("-verium"),
            CoinId::Vericoin => Some("-vericoin"),
        }
    }

    /// True when `getblockchaininfo.chain` matches the requested coin.
    fn rpc_chain_matches(self, rpc_chain: &str, cfg_chain: &str) -> bool {
        match self {
            CoinId::Vericoin => {
                rpc_chain == "vericoin" || rpc_chain == "binarytest-vericoin"
            }
            CoinId::Verium => {
                rpc_chain == "verium"
                    || rpc_chain == "binarytest-verium"
                    || rpc_chain == "main"
                    || (cfg_chain == "main" && rpc_chain == "main")
            }
        }
    }

    fn default_datadir(self) -> PathBuf {
        #[cfg(mobile)]
        {
            crate::config::app_config_base().join(match self {
                CoinId::Verium => "verium-chain",
                CoinId::Vericoin => "vericoin-chain",
            })
        }
        #[cfg(not(mobile))]
        {
            #[cfg(any(target_os = "windows", target_os = "macos"))]
            {
                if let Some(d) = dirs::data_dir() {
                    return match self {
                        CoinId::Verium => d.join("Verium"),
                        CoinId::Vericoin => d.join("Vericonomy"),
                    };
                }
            }
            #[cfg(not(any(target_os = "windows", target_os = "macos")))]
            {
                if let Some(h) = dirs::home_dir() {
                    return match self {
                        CoinId::Verium => h.join(".verium"),
                        CoinId::Vericoin => h.join(".vericonomy"),
                    };
                }
            }
            PathBuf::from(".")
        }
    }

    fn bootstrap_cdn_base(self) -> &'static str {
        match self {
            CoinId::Verium => "https://files.vericonomy.com/vrm/bootstrap",
            CoinId::Vericoin => "https://files.vericonomy.com/vrc/bootstrap",
        }
    }

    /// Base for wallet compatibility API on the production explorer-v2.
    fn explorer_api_base(self) -> &'static str {
        match self {
            CoinId::Verium => "https://explorer.vericonomy.com/v1/vrm/wallet",
            CoinId::Vericoin => "https://explorer.vericonomy.com/v1/vrc/wallet",
        }
    }

    /// Live chain API (`/blocks/latest`, `/block/:height`) on explorer-v2.
    fn explorer_chain_api_base(self) -> &'static str {
        match self {
            CoinId::Verium => "https://explorer.vericonomy.com/v1/vrm",
            CoinId::Vericoin => "https://explorer.vericonomy.com/v1/vrc",
        }
    }

    /// Indexer V2 chain id (`/api/indexer/:chainId/...`).
    fn explorer_indexer_chain_id(self) -> &'static str {
        match self {
            CoinId::Verium => "vrm",
            CoinId::Vericoin => "vrc",
        }
    }

    fn explorer_logo_url(self) -> &'static str {
        match self {
            CoinId::Verium => "https://explorer.vericonomy.com/img/vericonomy/verium-logo.svg",
            CoinId::Vericoin => {
                "https://explorer.vericonomy.com/img/vericonomy/vericoin-logo.svg"
            }
        }
    }

    fn confirmations_matured(self) -> u32 {
        self.profile().maturity_confirmations
    }

    fn earn_mode(self) -> &'static str {
        self.profile().earn_mode
    }

    fn keychain_service(self) -> String {
        format!("com.vericonomy.wallet.desktop.{}", self.as_str())
    }

    fn default_rpc_user(self) -> &'static str {
        match self {
            CoinId::Verium => "veriumwallet",
            CoinId::Vericoin => "vericoinwallet",
        }
    }

    fn wallet_backup_prefix(self) -> &'static str {
        match self {
            CoinId::Verium => "verium-wallet",
            CoinId::Vericoin => "vericoin-wallet",
        }
    }

    /// Default Electrum server URIs for light wallet mode (existing infrastructure).
    fn default_electrum_servers(self, network: NetworkMode) -> Vec<String> {
        self.profile().default_electrum_servers(network)
    }
}

// ---------------------------------------------------------------------------
// App-only CoinTarget helpers
// ---------------------------------------------------------------------------

pub trait CoinTargetAppExt {
    fn binarytest(coin: CoinId) -> Self;
    fn datadir(&self) -> PathBuf;
    fn extra_cli_args(&self) -> Vec<&'static str>;
    fn explorer_api_base(&self) -> Option<&'static str>;
    fn bootstrap_cdn_base(&self) -> Option<&'static str>;
    fn keychain_service(&self) -> String;
}

impl CoinTargetAppExt for CoinTarget {
    fn binarytest(coin: CoinId) -> Self {
        Self::new(coin, NetworkMode::BinaryTest)
    }

    /// Datadir subdirectory under the platform-default base. Binarytest gets
    /// the `binarytest-` prefix so it cannot collide with mainnet state.
    fn datadir(&self) -> PathBuf {
        let base_default = self.coin.default_datadir();
        match self.network {
            NetworkMode::Mainnet => base_default,
            NetworkMode::BinaryTest => {
                let dir_name = match self.coin {
                    CoinId::Verium => "binarytest-verium",
                    CoinId::Vericoin => "binarytest-vericoin",
                };
                if let Some(parent) = base_default.parent() {
                    parent.join(dir_name)
                } else {
                    PathBuf::from(dir_name)
                }
            }
        }
    }

    /// Extra CLI args the daemon needs. Binarytest requires `-binarytest`
    /// in addition to `-vericoin` / `-verium`.
    fn extra_cli_args(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        if self.network.is_test() {
            out.push("-binarytest");
        }
        match self.coin {
            CoinId::Verium => out.push("-verium"),
            CoinId::Vericoin => out.push("-vericoin"),
        }
        out
    }

    /// Suppress explorer URL when running on the binarytest network — there
    /// is no public explorer for binarytest. Callers should hide explorer
    /// links in this mode.
    fn explorer_api_base(&self) -> Option<&'static str> {
        match self.network {
            NetworkMode::Mainnet => Some(self.coin.explorer_api_base()),
            NetworkMode::BinaryTest => None,
        }
    }

    /// Suppress bootstrap CDN on binarytest — no canonical snapshot.
    fn bootstrap_cdn_base(&self) -> Option<&'static str> {
        match self.network {
            NetworkMode::Mainnet => Some(self.coin.bootstrap_cdn_base()),
            NetworkMode::BinaryTest => None,
        }
    }

    fn keychain_service(&self) -> String {
        match self.network {
            NetworkMode::Mainnet => self.coin.keychain_service(),
            NetworkMode::BinaryTest => {
                format!(
                    "com.vericonomy.wallet.desktop.binarytest.{}",
                    self.coin.as_str()
                )
            }
        }
    }
}


pub fn parse_coin_id(s: &str) -> AppResult<CoinId> {
    match s.trim().to_ascii_lowercase().as_str() {
        "verium" | "vrm" => Ok(CoinId::Verium),
        "vericoin" | "vrc" => Ok(CoinId::Vericoin),
        other => Err(AppError::other(format!("unknown coin: {other}"))),
    }
}

pub fn assert_verium(coin: CoinId) -> AppResult<()> {
    if coin != CoinId::Verium {
        return Err(AppError::other(format!(
            "command is only supported for Verium, not {}",
            coin.as_str()
        )));
    }
    Ok(())
}

pub fn assert_vericoin(coin: CoinId) -> AppResult<()> {
    if coin != CoinId::Vericoin {
        return Err(AppError::other(format!(
            "command is only supported for Vericoin, not {}",
            coin.as_str()
        )));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct CoinProfileSummary {
    pub id: String,
    pub symbol: String,
    pub display_name: String,
    pub tagline: String,
    pub earn_mode: String,
    pub default_rpc_port: u16,
    pub confirmations_matured: u32,
}

pub fn profile_summary(coin: CoinId) -> CoinProfileSummary {
    use CoinIdAppExt as _;
    let tagline = match coin {
        CoinId::Verium => "Reserve",
        CoinId::Vericoin => "Currency",
    };
    CoinProfileSummary {
        id: coin.as_str().to_string(),
        symbol: coin.symbol().to_string(),
        display_name: coin.display_name().to_string(),
        tagline: tagline.to_string(),
        earn_mode: coin.earn_mode().to_string(),
        default_rpc_port: coin.default_rpc_port(),
        confirmations_matured: coin.confirmations_matured(),
    }
}

pub fn all_profile_summaries() -> Vec<CoinProfileSummary> {
    CoinId::all()
        .iter()
        .copied()
        .map(profile_summary)
        .collect()
}

pub fn coin_map<T: Clone>(value_fn: impl Fn(CoinId) -> T) -> HashMap<String, T> {
    CoinId::all()
        .iter()
        .map(|coin| (coin.as_str().to_string(), value_fn(*coin)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpc_chain_match_vericoin() {
        use CoinIdAppExt as _;
        assert!(CoinId::Vericoin.rpc_chain_matches("vericoin", "vericoin"));
        assert!(CoinId::Vericoin.rpc_chain_matches("binarytest-vericoin", "binarytest-vericoin"));
        assert!(!CoinId::Vericoin.rpc_chain_matches("verium", "vericoin"));
        assert!(!CoinId::Vericoin.rpc_chain_matches("main", "vericoin"));
    }

    #[test]
    fn rpc_chain_match_verium() {
        assert!(CoinId::Verium.rpc_chain_matches("verium", "main"));
        assert!(CoinId::Verium.rpc_chain_matches("main", "main"));
        assert!(CoinId::Verium.rpc_chain_matches("binarytest-verium", "binarytest-verium"));
        assert!(!CoinId::Verium.rpc_chain_matches("vericoin", "main"));
    }
}
