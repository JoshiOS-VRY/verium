//! User preferences — compatible with Tauri `prefs.json`.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use vericonomy_wallet_engine::WalletMode;

use crate::coin::CoinId;
use crate::error::{HostError, HostResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    #[serde(default)]
    pub setup_completed: bool,
    #[serde(default)]
    pub setup_completed_by_coin: Option<HashMap<String, bool>>,
    #[serde(default = "default_active_coin")]
    pub active_coin: String,
    #[serde(default = "default_true")]
    pub verium_enabled: bool,
    #[serde(default = "default_true")]
    pub vericoin_enabled: bool,
    #[serde(default)]
    pub auto_mine_on_open: bool,
    #[serde(default)]
    pub auto_stake_on_open: bool,
    #[serde(default = "default_auto_mine_threads")]
    pub auto_mine_threads: u32,
    #[serde(default = "default_true")]
    pub auto_adjust_mine_threads: bool,
    #[serde(default = "default_theme_mode")]
    pub theme_mode: String,
    #[serde(default)]
    pub wallet_mode: WalletMode,
    #[serde(default)]
    pub wallet_mode_by_coin: Option<HashMap<String, WalletMode>>,
    #[serde(default)]
    pub electrum_servers_by_coin: Option<HashMap<String, Vec<String>>>,
    /// Preserve unknown Tauri fields so Qt does not strip prefs on save.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            setup_completed: false,
            setup_completed_by_coin: None,
            active_coin: default_active_coin(),
            verium_enabled: true,
            vericoin_enabled: true,
            auto_mine_on_open: false,
            auto_stake_on_open: false,
            auto_mine_threads: default_auto_mine_threads(),
            auto_adjust_mine_threads: true,
            theme_mode: default_theme_mode(),
            wallet_mode: WalletMode::FullNode,
            wallet_mode_by_coin: None,
            electrum_servers_by_coin: None,
            extra: HashMap::new(),
        }
    }
}

fn default_active_coin() -> String {
    "verium".into()
}
fn default_true() -> bool {
    true
}
fn default_auto_mine_threads() -> u32 {
    2
}
fn default_theme_mode() -> String {
    "dark".into()
}

pub fn prefs_path() -> PathBuf {
    crate::config::app_config_base().join("prefs.json")
}

fn legacy_prefs_path() -> PathBuf {
    let base = dirs::config_dir()
        .or_else(dirs::data_dir)
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("Verium").join("desktop-app").join("prefs.json")
}

pub fn wallet_mode_for(prefs: &UserPreferences, coin: CoinId) -> WalletMode {
    prefs
        .wallet_mode_by_coin
        .as_ref()
        .and_then(|m| m.get(coin.as_str()).copied())
        .unwrap_or(prefs.wallet_mode)
}

pub fn set_wallet_mode_for(prefs: &mut UserPreferences, coin: CoinId, mode: WalletMode) {
    let mut map = prefs.wallet_mode_by_coin.take().unwrap_or_default();
    map.insert(coin.as_str().to_string(), mode);
    prefs.wallet_mode_by_coin = Some(map);
}

pub fn electrum_servers_for(prefs: &UserPreferences, coin: CoinId) -> Vec<String> {
    use vericonomy_chain_params::NetworkMode;
    let sdk = coin.to_sdk();
    prefs
        .electrum_servers_by_coin
        .as_ref()
        .and_then(|m| m.get(coin.as_str()).cloned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            sdk.profile()
                .default_electrum_servers(NetworkMode::Mainnet)
                .into_iter()
                .map(String::from)
                .collect()
        })
}

pub fn load_prefs() -> HostResult<UserPreferences> {
    let path = prefs_path();
    let mut prefs = if path.exists() {
        let text = std::fs::read_to_string(&path)?;
        serde_json::from_str(&text).unwrap_or_default()
    } else {
        let legacy = legacy_prefs_path();
        if legacy.exists() {
            let text = std::fs::read_to_string(&legacy)?;
            serde_json::from_str(&text).unwrap_or_default()
        } else {
            UserPreferences::default()
        }
    };
    if reconcile_prefs_on_load(&mut prefs)? {
        save_prefs(&prefs)?;
    }
    Ok(prefs)
}

/// Keep full-node as the desktop default. Light mode is only active when the user
/// sets an explicit per-coin override in Settings (`wallet_mode_by_coin`).
pub fn reconcile_prefs_on_load(prefs: &mut UserPreferences) -> HostResult<bool> {
    let mut changed = false;

    // App-wide default is always full node for the Qt desktop shell.
    if prefs.wallet_mode.is_light() {
        prefs.wallet_mode = WalletMode::FullNode;
        changed = true;
    }

    for coin in [CoinId::Verium, CoinId::Vericoin] {
        let explicit = prefs
            .wallet_mode_by_coin
            .as_ref()
            .and_then(|m| m.get(coin.as_str()).copied());

        // Only honor light mode when the user explicitly chose it per coin.
        if wallet_mode_for(prefs, coin).is_light() && explicit != Some(WalletMode::Light) {
            set_wallet_mode_for(prefs, coin, WalletMode::FullNode);
            changed = true;
        }
    }

    Ok(changed)
}

pub fn activate_light_mode_for(coin: CoinId) -> HostResult<()> {
    let mut prefs = load_prefs()?;
    set_wallet_mode_for(&mut prefs, coin, WalletMode::Light);
    save_prefs(&prefs)
}

pub fn save_prefs(prefs: &UserPreferences) -> HostResult<()> {
    let path = prefs_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(prefs)?)?;
    Ok(())
}

pub trait CoinSdkExt {
    fn to_sdk(self) -> vericonomy_chain_params::CoinId;
}

impl CoinSdkExt for CoinId {
    fn to_sdk(self) -> vericonomy_chain_params::CoinId {
        match self {
            CoinId::Verium => vericonomy_chain_params::CoinId::Verium,
            CoinId::Vericoin => vericonomy_chain_params::CoinId::Vericoin,
        }
    }
}

pub fn map_wallet_err(e: vericonomy_errors::WalletError) -> HostError {
    HostError::Other(e.to_string())
}
