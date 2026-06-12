use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::coin_profile::{CoinId, NetworkMode};
use crate::config::{load_config_for_network, resolve_legacy_wallet_outside_cfg, wallet_dat_exists};
use crate::onboarding::OnboardingCheckpoint;
use crate::wallet::keystore;
use crate::wallet::mode::WalletMode;
use crate::config::app_config_base;
use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    #[serde(default)]
    pub setup_completed: bool,
    /// First-run wizard finished per chain (`verium`, `vericoin`).
    #[serde(default)]
    pub setup_completed_by_coin: Option<HashMap<String, bool>>,
    #[serde(default)]
    pub bootstrap_dismissed_at: Option<i64>,
    #[serde(default = "default_active_coin")]
    pub active_coin: String,
    #[serde(default = "default_true")]
    pub verium_enabled: bool,
    #[serde(default = "default_true")]
    pub vericoin_enabled: bool,
    #[serde(default)]
    pub explorer_tx_url_template: String,
    #[serde(default)]
    pub explorer_block_url_template: Option<String>,
    #[serde(default)]
    pub explorer_address_url_template: Option<String>,
    /// Start CPU mining automatically when the app opens (requires unlocked wallet).
    #[serde(default)]
    pub auto_mine_on_open: bool,
    /// Start staking automatically when the app opens (Vericoin).
    #[serde(default)]
    pub auto_stake_on_open: bool,
    /// Play a chime when the wallet receives a new coinbase (block found).
    #[serde(default)]
    pub play_sound_on_block_mined: bool,
    /// Play a chime when the wallet mints a stake reward (Vericoin).
    #[serde(default)]
    pub play_sound_on_stake_reward: bool,
    /// Toast + chime when incoming VRM is received while the app is open.
    #[serde(default = "default_notify_on_vrm_received")]
    pub notify_on_vrm_received: bool,
    /// Toast + chime when incoming VRC is received while the app is open.
    #[serde(default = "default_notify_on_vrc_received")]
    pub notify_on_vrc_received: bool,
    #[serde(default = "default_auto_mine_threads")]
    pub auto_mine_threads: u32,
    /// When true, thread count follows CPU topology; when false, uses auto_mine_threads.
    #[serde(default = "default_auto_adjust_mine_threads")]
    pub auto_adjust_mine_threads: bool,
    /// "dynamic" (default) or "static" — how block rewards choose a payout address.
    #[serde(default = "default_mining_reward_address_mode")]
    pub mining_reward_address_mode: String,
    /// Wallet address for block rewards when mining_reward_address_mode is "static".
    #[serde(default)]
    pub mining_reward_address: Option<String>,
    #[serde(default)]
    pub pause_mine_on_battery: bool,
    #[serde(default = "default_mine_core_affinity")]
    pub mine_core_affinity: String,
    #[serde(default)]
    pub mining_power_watts: Option<f64>,
    #[serde(default)]
    pub mining_cost_per_kwh: Option<f64>,
    /// Optional VRM/USD price assumption for solo revenue estimates.
    #[serde(default)]
    pub mining_vrm_price_usd: Option<f64>,
    /// VRM address for public pool payouts (Stratum username prefix).
    #[serde(default)]
    pub pool_payout_address: Option<String>,
    /// Worker suffix for public pool mining (`ADDRESS.worker`).
    #[serde(default)]
    pub pool_worker_name: Option<String>,
    /// Last selected mining tab: `solo` or `pool`.
    #[serde(default)]
    pub mining_mode: Option<String>,
    /// "system" (follow OS), "light", or "dark". Defaults to "system".
    #[serde(default = "default_theme_mode")]
    pub theme_mode: String,
    /// Default unlock duration (seconds) when no per-coin override is set.
    #[serde(default = "default_wallet_unlock_duration")]
    pub wallet_unlock_duration_seconds: u32,
    /// Optional per-coin unlock durations keyed by coin id (`verium`, `vericoin`).
    #[serde(default)]
    pub wallet_unlock_duration_by_coin: Option<HashMap<String, u32>>,
    /// Custom on-chain fee rate in VRM/kB. None falls back to the daemon default.
    #[serde(default)]
    pub tx_fee_rate_vrm_per_kb: Option<f64>,
    /// Unix timestamp when bootstrap was last imported, keyed by coin id.
    #[serde(default)]
    pub bootstrap_imported_at_by_coin: Option<HashMap<String, i64>>,
    /// Which physical network the wallet is operating against. Defaults to
    /// Mainnet. BinaryTest is the isolated Binary Chain v3 (DACE) test
    /// network — see vericoin/doc/dace/binarytest-network.md. Switching
    /// modes restarts the daemons and clears RPC URL overrides.
    #[serde(default)]
    pub network_mode: NetworkMode,
    /// App-wide default: full local node vs Electrum light client. Retained as
    /// the fallback when a coin has no explicit `wallet_mode_by_coin` entry, so
    /// old prefs keep working and new installs inherit one sensible default.
    #[serde(default)]
    pub wallet_mode: WalletMode,
    /// Per-coin wallet mode overrides keyed by coin id (`verium`, `vericoin`).
    /// Enables e.g. Verium full node + Vericoin light on the same device.
    #[serde(default)]
    pub wallet_mode_by_coin: Option<HashMap<String, WalletMode>>,
    /// Optional per-coin Electrum server URIs (`host:port` or `tls://host:port`).
    #[serde(default)]
    pub electrum_servers_by_coin: Option<HashMap<String, Vec<String>>>,
    /// Resumable onboarding checkpoints keyed by coin id. Lets the setup wizard
    /// survive refreshes and restarts without losing the user's place.
    #[serde(default)]
    pub onboarding_by_coin: Option<HashMap<String, OnboardingCheckpoint>>,
    /// Mobile: unlock light wallet with Face ID / Touch ID after storing passphrase in keychain.
    #[serde(default)]
    pub biometric_unlock_enabled: bool,
}

fn default_active_coin() -> String {
    "verium".to_string()
}

fn default_true() -> bool {
    true
}

fn default_auto_mine_threads() -> u32 {
    2
}

fn default_auto_adjust_mine_threads() -> bool {
    true
}

fn default_mining_reward_address_mode() -> String {
    "dynamic".to_string()
}

fn default_mine_core_affinity() -> String {
    "performance".to_string()
}

fn default_theme_mode() -> String {
    "system".to_string()
}

fn default_wallet_unlock_duration() -> u32 {
    4 * 60 * 60
}

fn default_notify_on_vrm_received() -> bool {
    true
}

fn default_notify_on_vrc_received() -> bool {
    true
}

fn default_tx_template() -> String {
    "https://explorer.vericonomy.com/vrm/tx/%s".to_string()
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            setup_completed: false,
            setup_completed_by_coin: None,
            bootstrap_dismissed_at: None,
            active_coin: default_active_coin(),
            verium_enabled: true,
            vericoin_enabled: true,
            explorer_tx_url_template: default_tx_template(),
            explorer_block_url_template: None,
            explorer_address_url_template: None,
            auto_mine_on_open: false,
            auto_stake_on_open: false,
            play_sound_on_block_mined: false,
            play_sound_on_stake_reward: false,
            notify_on_vrm_received: default_notify_on_vrm_received(),
            notify_on_vrc_received: default_notify_on_vrc_received(),
            auto_mine_threads: default_auto_mine_threads(),
            auto_adjust_mine_threads: default_auto_adjust_mine_threads(),
            mining_reward_address_mode: default_mining_reward_address_mode(),
            mining_reward_address: None,
            pause_mine_on_battery: false,
            mine_core_affinity: default_mine_core_affinity(),
            mining_power_watts: None,
            mining_cost_per_kwh: None,
            mining_vrm_price_usd: None,
            pool_payout_address: None,
            pool_worker_name: None,
            mining_mode: None,
            theme_mode: default_theme_mode(),
            wallet_unlock_duration_seconds: default_wallet_unlock_duration(),
            wallet_unlock_duration_by_coin: None,
            tx_fee_rate_vrm_per_kb: None,
            bootstrap_imported_at_by_coin: None,
            network_mode: NetworkMode::Mainnet,
            wallet_mode: WalletMode::FullNode,
            wallet_mode_by_coin: None,
            electrum_servers_by_coin: None,
            onboarding_by_coin: None,
            biometric_unlock_enabled: false,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct PartialUserPreferences {
    pub setup_completed: Option<bool>,
    pub setup_completed_by_coin: Option<HashMap<String, bool>>,
    pub bootstrap_dismissed_at: Option<i64>,
    pub active_coin: Option<String>,
    pub verium_enabled: Option<bool>,
    pub vericoin_enabled: Option<bool>,
    pub explorer_tx_url_template: Option<String>,
    pub explorer_block_url_template: Option<String>,
    pub explorer_address_url_template: Option<String>,
    pub auto_mine_on_open: Option<bool>,
    pub auto_stake_on_open: Option<bool>,
    pub play_sound_on_block_mined: Option<bool>,
    pub play_sound_on_stake_reward: Option<bool>,
    pub notify_on_vrm_received: Option<bool>,
    pub notify_on_vrc_received: Option<bool>,
    pub auto_mine_threads: Option<u32>,
    pub auto_adjust_mine_threads: Option<bool>,
    pub mining_reward_address_mode: Option<String>,
    pub mining_reward_address: Option<String>,
    pub pause_mine_on_battery: Option<bool>,
    pub mine_core_affinity: Option<String>,
    pub mining_power_watts: Option<f64>,
    pub mining_cost_per_kwh: Option<f64>,
    pub mining_vrm_price_usd: Option<f64>,
    pub pool_payout_address: Option<String>,
    pub pool_worker_name: Option<String>,
    pub mining_mode: Option<String>,
    pub theme_mode: Option<String>,
    pub wallet_unlock_duration_seconds: Option<u32>,
    pub wallet_unlock_duration_by_coin: Option<HashMap<String, u32>>,
    pub tx_fee_rate_vrm_per_kb: Option<f64>,
    pub bootstrap_imported_at_by_coin: Option<HashMap<String, i64>>,
    pub network_mode: Option<NetworkMode>,
    pub wallet_mode: Option<WalletMode>,
    pub wallet_mode_by_coin: Option<HashMap<String, WalletMode>>,
    pub electrum_servers_by_coin: Option<HashMap<String, Vec<String>>>,
    pub onboarding_by_coin: Option<HashMap<String, OnboardingCheckpoint>>,
    pub biometric_unlock_enabled: Option<bool>,
}

pub fn prefs_path() -> PathBuf {
    app_config_base().join("prefs.json")
}

fn legacy_prefs_path() -> PathBuf {
    let base = if let Some(d) = dirs::config_dir() {
        d
    } else if let Some(d) = dirs::data_dir() {
        d
    } else if let Some(d) = dirs::home_dir() {
        d
    } else {
        PathBuf::from(".")
    };
    base.join("Verium").join("desktop-app").join("prefs.json")
}

pub fn coin_enabled(prefs: &UserPreferences, coin: CoinId) -> bool {
    match coin {
        CoinId::Verium => prefs.verium_enabled,
        CoinId::Vericoin => prefs.vericoin_enabled,
    }
}

/// Effective wallet mode for one coin: explicit per-coin override, else the
/// app-wide `wallet_mode` default. This is the single source of truth — call
/// sites must use this rather than reading `prefs.wallet_mode` directly.
pub fn wallet_mode_for(prefs: &UserPreferences, coin: CoinId) -> WalletMode {
    prefs
        .wallet_mode_by_coin
        .as_ref()
        .and_then(|m| m.get(coin.as_str()).copied())
        .unwrap_or(prefs.wallet_mode)
}

/// Persist a per-coin wallet mode override (mutates in place; caller saves).
pub fn set_wallet_mode_for(prefs: &mut UserPreferences, coin: CoinId, mode: WalletMode) {
    let mut map = prefs.wallet_mode_by_coin.take().unwrap_or_default();
    map.insert(coin.as_str().to_string(), mode);
    prefs.wallet_mode_by_coin = Some(map);
}

/// Effective Electrum server list for one coin: per-coin prefs, else chain defaults.
/// Empty stored lists are ignored so a partial prefs migration cannot leave VRC with
/// zero servers while VRM is configured.
pub fn electrum_servers_for(prefs: &UserPreferences, coin: CoinId) -> Vec<String> {
    let network = crate::features::effective_network_mode(prefs.network_mode);
    prefs
        .electrum_servers_by_coin
        .as_ref()
        .and_then(|m| m.get(coin.as_str()).cloned())
        .filter(|servers| !servers.is_empty())
        .unwrap_or_else(|| coin.default_electrum_servers(network))
}

/// Seed default Electrum endpoints for every enabled chain when prefs omit them.
pub fn reconcile_electrum_servers(prefs: &mut UserPreferences) -> bool {
    let network = crate::features::effective_network_mode(prefs.network_mode);
    let mut changed = false;
    let mut map = prefs.electrum_servers_by_coin.clone().unwrap_or_default();

    for coin in CoinId::all() {
        if !coin_enabled(prefs, *coin) {
            continue;
        }
        let key = coin.as_str().to_string();
        let needs_defaults = map.get(&key).map(|servers| servers.is_empty()).unwrap_or(true);
        if needs_defaults {
            map.insert(key, coin.default_electrum_servers(network));
            changed = true;
        }
    }

    if changed {
        prefs.electrum_servers_by_coin = Some(map);
    }
    changed
}

/// Read the onboarding checkpoint for a coin (default = not started).
pub fn onboarding_for(prefs: &UserPreferences, coin: CoinId) -> OnboardingCheckpoint {
    prefs
        .onboarding_by_coin
        .as_ref()
        .and_then(|m| m.get(coin.as_str()).cloned())
        .unwrap_or_default()
}

/// Persist an onboarding checkpoint for a coin (mutates in place; caller saves).
pub fn set_onboarding_for(
    prefs: &mut UserPreferences,
    coin: CoinId,
    checkpoint: OnboardingCheckpoint,
) {
    let mut map = prefs.onboarding_by_coin.take().unwrap_or_default();
    map.insert(coin.as_str().to_string(), checkpoint);
    prefs.onboarding_by_coin = Some(map);
}

pub fn wallet_unlock_duration_for(prefs: &UserPreferences, coin: CoinId) -> u32 {
    prefs
        .wallet_unlock_duration_by_coin
        .as_ref()
        .and_then(|m| m.get(coin.as_str()).copied())
        .unwrap_or(prefs.wallet_unlock_duration_seconds)
}

const PREFS_STORE_LABEL: &str = "user-preferences";

/// How long a loaded `UserPreferences` is reused before re-reading from disk.
/// `load()` is called on nearly every wallet-service call and every
/// supervisor/heal/watcher tick; decrypting + parsing + keystore reconciliation
/// each time is wasteful. The cache is invalidated on every save so writes are
/// always reflected immediately.
const PREFS_CACHE_TTL: Duration = Duration::from_secs(3);

static PREFS_CACHE: Lazy<Mutex<Option<(Instant, UserPreferences)>>> =
    Lazy::new(|| Mutex::new(None));

fn cached_prefs() -> Option<UserPreferences> {
    let guard = PREFS_CACHE.lock().ok()?;
    let (at, prefs) = guard.as_ref()?;
    if at.elapsed() < PREFS_CACHE_TTL {
        Some(prefs.clone())
    } else {
        None
    }
}

fn store_prefs_cache(prefs: &UserPreferences) {
    if let Ok(mut guard) = PREFS_CACHE.lock() {
        *guard = Some((Instant::now(), prefs.clone()));
    }
}

/// Drop the cached preferences so the next `load()` re-reads from disk.
pub fn invalidate_prefs_cache() {
    if let Ok(mut guard) = PREFS_CACHE.lock() {
        *guard = None;
    }
}

/// Mark setup complete for chains that already have a persisted light keystore.
pub fn reconcile_setup_flags_with_keystore(prefs: &mut UserPreferences) -> AppResult<bool> {
    let mut changed = false;
    let mut setup = prefs.setup_completed_by_coin.clone().unwrap_or_default();

    for coin in [CoinId::Verium, CoinId::Vericoin] {
        if keystore::light_wallet_on_disk(coin)
            && setup.get(coin.as_str()) != Some(&true)
        {
            setup.insert(coin.as_str().to_string(), true);
            changed = true;
        }
    }

    if changed {
        prefs.setup_completed_by_coin = Some(setup.clone());
        if setup.get(CoinId::Verium.as_str()) == Some(&true) {
            prefs.setup_completed = true;
        }
    }

    Ok(changed)
}

/// Align setup flags and per-coin wallet mode with on-disk light keystores
/// (e.g. after an import). Sets a coin to light mode when it has a light
/// keystore but no full-node `wallet.dat` and no explicit full-node override.
fn reconcile_prefs_with_keystore(prefs: &mut UserPreferences) -> AppResult<bool> {
    let mut changed = reconcile_setup_flags_with_keystore(prefs)?;
    changed |= reconcile_electrum_servers(prefs);

    for coin in [CoinId::Verium, CoinId::Vericoin] {
        if wallet_mode_for(prefs, coin).is_light() {
            continue;
        }
        // Only auto-flip coins that have no explicit per-coin override yet.
        let has_explicit_override = prefs
            .wallet_mode_by_coin
            .as_ref()
            .map(|m| m.contains_key(coin.as_str()))
            .unwrap_or(false);
        if has_explicit_override {
            continue;
        }
        if !keystore::light_wallet_on_disk(coin) {
            continue;
        }
        if let Ok(cfg) = load_config_for_network(coin, prefs.network_mode) {
            let has_full_node_wallet = wallet_dat_exists(coin, &cfg)
                || resolve_legacy_wallet_outside_cfg(coin, &cfg).is_some();
            if !has_full_node_wallet {
                set_wallet_mode_for(prefs, coin, WalletMode::Light);
                changed = true;
                tracing::info!(
                    "wallet_mode[{}] set to light: light wallet exists without wallet.dat",
                    coin.as_str()
                );
            }
        }
    }

    // Inherited app-wide light default: flip to full_node when only wallet.dat
    // exists. Skip coins with an explicit per-coin light override (mid-setup or
    // completed light wallet on a chain that also has a full node).
    for coin in [CoinId::Verium, CoinId::Vericoin] {
        if !wallet_mode_for(prefs, coin).is_light() {
            continue;
        }
        let explicit_light = prefs
            .wallet_mode_by_coin
            .as_ref()
            .and_then(|m| m.get(coin.as_str()))
            .map(|m| m.is_light())
            .unwrap_or(false);
        if explicit_light {
            continue;
        }
        if keystore::light_wallet_on_disk(coin) {
            continue;
        }
        if let Ok(cfg) = load_config_for_network(coin, prefs.network_mode) {
            let has_full_node_wallet = wallet_dat_exists(coin, &cfg)
                || resolve_legacy_wallet_outside_cfg(coin, &cfg).is_some();
            if has_full_node_wallet {
                set_wallet_mode_for(prefs, coin, WalletMode::FullNode);
                changed = true;
                tracing::info!(
                    "wallet_mode[{}] set to full_node: inherited light default but only wallet.dat exists",
                    coin.as_str()
                );
            }
        }
    }

    Ok(changed)
}

/// Load preferences without blocking on the async runtime (safe from Tauri setup and sync commands).
pub fn load_sync() -> AppResult<UserPreferences> {
    if let Some(prefs) = cached_prefs() {
        return Ok(prefs);
    }
    let legacy = legacy_prefs_path();
    let path = prefs_path();
    if legacy.exists() && !crate::secret_store::blob_exists(PREFS_STORE_LABEL) {
        let raw = fs::read_to_string(&legacy)?;
        if let Ok(prefs) = serde_json::from_str::<UserPreferences>(&raw) {
            if let Err(e) = save_sync(&prefs) {
                tracing::warn!("could not persist migrated legacy prefs: {e}");
            } else {
                let _ = fs::remove_file(&legacy);
            }
            store_prefs_cache(&prefs);
            return Ok(prefs);
        }
    }

    if path.exists() && !crate::secret_store::blob_readable(PREFS_STORE_LABEL) {
        let raw = fs::read_to_string(&path)?;
        if let Ok(prefs) = serde_json::from_str::<UserPreferences>(&raw) {
            if let Err(e) = save_sync(&prefs) {
                tracing::warn!("could not re-seal prefs from plaintext: {e}");
            }
            store_prefs_cache(&prefs);
            return Ok(prefs);
        }
    }

    crate::secret_store::migrate_plaintext_json(PREFS_STORE_LABEL, &path)?;
    let mut prefs = crate::secret_store::load_json(
        PREFS_STORE_LABEL,
        &path,
        UserPreferences::default(),
    )?;

    if path.exists() {
        if let Ok(raw) = fs::read_to_string(&path) {
            if let Ok(plain) = serde_json::from_str::<UserPreferences>(&raw) {
                if plain.network_mode != prefs.network_mode {
                    prefs.network_mode = plain.network_mode;
                }
            }
        }
    }

    if prefs.setup_completed {
        let mut m = prefs.setup_completed_by_coin.clone().unwrap_or_default();
        m.entry(CoinId::Verium.as_str().to_string()).or_insert(true);
        prefs.setup_completed_by_coin = Some(m);
    }

    if reconcile_prefs_with_keystore(&mut prefs)? {
        if let Err(e) = save_sync(&prefs) {
            tracing::warn!(
                "prefs reconcile changed flags but could not persist (continuing with in-memory prefs): {e}"
            );
        }
    }

    store_prefs_cache(&prefs);
    Ok(prefs)
}

fn save_sync(prefs: &UserPreferences) -> AppResult<()> {
    let path = prefs_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(prefs)?;
    fs::write(&path, json)?;
    if crate::secret_store::encrypted_data_orphaned() {
        tracing::warn!("prefs saved to plaintext only: Windows Credential Manager entry missing");
    } else if let Err(e) = crate::secret_store::save_json(PREFS_STORE_LABEL, prefs) {
        tracing::warn!("could not save encrypted prefs: {e}");
    }
    store_prefs_cache(prefs);
    Ok(())
}

pub async fn load() -> AppResult<UserPreferences> {
    load_sync()
}

pub async fn save(prefs: &UserPreferences) -> AppResult<()> {
    save_sync(prefs)
}

#[cfg(test)]
mod mode_tests {
    use super::*;

    #[test]
    fn per_coin_mode_falls_back_to_app_wide_default() {
        let mut prefs = UserPreferences::default();
        prefs.wallet_mode = WalletMode::FullNode;
        assert_eq!(wallet_mode_for(&prefs, CoinId::Verium), WalletMode::FullNode);
        assert_eq!(wallet_mode_for(&prefs, CoinId::Vericoin), WalletMode::FullNode);
    }

    #[test]
    fn per_coin_override_wins_independently() {
        let mut prefs = UserPreferences::default();
        prefs.wallet_mode = WalletMode::FullNode;
        set_wallet_mode_for(&mut prefs, CoinId::Vericoin, WalletMode::Light);
        // Vericoin flips to light; Verium keeps the full-node default.
        assert_eq!(wallet_mode_for(&prefs, CoinId::Vericoin), WalletMode::Light);
        assert_eq!(wallet_mode_for(&prefs, CoinId::Verium), WalletMode::FullNode);
    }

    #[test]
    fn onboarding_checkpoint_round_trips() {
        let mut prefs = UserPreferences::default();
        let mut cp = onboarding_for(&prefs, CoinId::Verium);
        assert_eq!(cp.phase, crate::onboarding::OnboardingPhase::NotStarted);
        cp.step = Some("wallet".to_string());
        set_onboarding_for(&mut prefs, CoinId::Verium, cp);
        assert_eq!(
            onboarding_for(&prefs, CoinId::Verium).step.as_deref(),
            Some("wallet")
        );
    }

    #[test]
    fn electrum_servers_for_ignores_empty_stored_list() {
        let mut prefs = UserPreferences::default();
        prefs.electrum_servers_by_coin = Some(HashMap::from([
            (
                CoinId::Vericoin.as_str().to_string(),
                Vec::<String>::new(),
            ),
        ]));
        let servers = electrum_servers_for(&prefs, CoinId::Vericoin);
        assert!(
            servers.iter().any(|s| s.contains("vrc")),
            "expected VRC defaults when stored list is empty: {servers:?}"
        );
    }

    #[test]
    fn reconcile_electrum_servers_seeds_both_chains() {
        let mut prefs = UserPreferences::default();
        assert!(reconcile_electrum_servers(&mut prefs));
        let map = prefs.electrum_servers_by_coin.expect("map");
        let vrm = map.get(CoinId::Verium.as_str()).expect("vrm");
        let vrc = map.get(CoinId::Vericoin.as_str()).expect("vrc");
        assert!(vrm.iter().any(|s| s.contains("vrm")));
        assert!(vrc.iter().any(|s| s.contains("vrc")));
    }

    #[test]
    fn reconcile_setup_flags_unchanged_without_keystore_wallets() {
        let mut prefs = UserPreferences::default();
        assert!(
            !reconcile_setup_flags_with_keystore(&mut prefs).expect("reconcile")
        );
        assert!(
            prefs
                .setup_completed_by_coin
                .unwrap_or_default()
                .get(CoinId::Verium.as_str())
                .is_none()
        );
    }
}

pub fn merge(current: UserPreferences, partial: PartialUserPreferences) -> UserPreferences {
    UserPreferences {
        setup_completed: partial.setup_completed.unwrap_or(current.setup_completed),
        setup_completed_by_coin: partial
            .setup_completed_by_coin
            .or(current.setup_completed_by_coin),
        bootstrap_dismissed_at: partial
            .bootstrap_dismissed_at
            .or(current.bootstrap_dismissed_at),
        active_coin: partial.active_coin.unwrap_or(current.active_coin),
        verium_enabled: partial.verium_enabled.unwrap_or(current.verium_enabled),
        vericoin_enabled: partial.vericoin_enabled.unwrap_or(current.vericoin_enabled),
        explorer_tx_url_template: partial
            .explorer_tx_url_template
            .unwrap_or(current.explorer_tx_url_template),
        explorer_block_url_template: partial
            .explorer_block_url_template
            .or(current.explorer_block_url_template),
        explorer_address_url_template: partial
            .explorer_address_url_template
            .or(current.explorer_address_url_template),
        auto_mine_on_open: partial.auto_mine_on_open.unwrap_or(current.auto_mine_on_open),
        auto_stake_on_open: partial.auto_stake_on_open.unwrap_or(current.auto_stake_on_open),
        play_sound_on_block_mined: partial
            .play_sound_on_block_mined
            .unwrap_or(current.play_sound_on_block_mined),
        play_sound_on_stake_reward: partial
            .play_sound_on_stake_reward
            .unwrap_or(current.play_sound_on_stake_reward),
        notify_on_vrm_received: partial
            .notify_on_vrm_received
            .unwrap_or(current.notify_on_vrm_received),
        notify_on_vrc_received: partial
            .notify_on_vrc_received
            .unwrap_or(current.notify_on_vrc_received),
        auto_mine_threads: partial.auto_mine_threads.unwrap_or(current.auto_mine_threads),
        auto_adjust_mine_threads: partial
            .auto_adjust_mine_threads
            .unwrap_or(current.auto_adjust_mine_threads),
        mining_reward_address_mode: partial
            .mining_reward_address_mode
            .unwrap_or(current.mining_reward_address_mode),
        mining_reward_address: partial
            .mining_reward_address
            .or(current.mining_reward_address),
        pause_mine_on_battery: partial
            .pause_mine_on_battery
            .unwrap_or(current.pause_mine_on_battery),
        mine_core_affinity: partial
            .mine_core_affinity
            .unwrap_or(current.mine_core_affinity),
        mining_power_watts: partial.mining_power_watts.or(current.mining_power_watts),
        mining_cost_per_kwh: partial.mining_cost_per_kwh.or(current.mining_cost_per_kwh),
        mining_vrm_price_usd: partial
            .mining_vrm_price_usd
            .or(current.mining_vrm_price_usd),
        pool_payout_address: partial
            .pool_payout_address
            .or(current.pool_payout_address),
        pool_worker_name: partial.pool_worker_name.or(current.pool_worker_name),
        mining_mode: partial.mining_mode.or(current.mining_mode),
        theme_mode: partial.theme_mode.unwrap_or(current.theme_mode),
        wallet_unlock_duration_seconds: partial
            .wallet_unlock_duration_seconds
            .unwrap_or(current.wallet_unlock_duration_seconds),
        wallet_unlock_duration_by_coin: {
            let mut merged = current
                .wallet_unlock_duration_by_coin
                .clone()
                .unwrap_or_default();
            if let Some(partial_map) = partial.wallet_unlock_duration_by_coin {
                merged.extend(partial_map);
            }
            if merged.is_empty() {
                None
            } else {
                Some(merged)
            }
        },
        tx_fee_rate_vrm_per_kb: partial
            .tx_fee_rate_vrm_per_kb
            .or(current.tx_fee_rate_vrm_per_kb),
        bootstrap_imported_at_by_coin: {
            let mut merged = current
                .bootstrap_imported_at_by_coin
                .clone()
                .unwrap_or_default();
            if let Some(partial_map) = partial.bootstrap_imported_at_by_coin {
                merged.extend(partial_map);
            }
            if merged.is_empty() {
                None
            } else {
                Some(merged)
            }
        },
        network_mode: partial.network_mode.unwrap_or(current.network_mode),
        wallet_mode: partial.wallet_mode.unwrap_or(current.wallet_mode),
        wallet_mode_by_coin: {
            let mut merged = current.wallet_mode_by_coin.clone().unwrap_or_default();
            if let Some(partial_map) = partial.wallet_mode_by_coin {
                merged.extend(partial_map);
            }
            if merged.is_empty() {
                None
            } else {
                Some(merged)
            }
        },
        electrum_servers_by_coin: partial
            .electrum_servers_by_coin
            .or(current.electrum_servers_by_coin),
        onboarding_by_coin: {
            let mut merged = current.onboarding_by_coin.clone().unwrap_or_default();
            if let Some(partial_map) = partial.onboarding_by_coin {
                merged.extend(partial_map);
            }
            if merged.is_empty() {
                None
            } else {
                Some(merged)
            }
        },
        biometric_unlock_enabled: partial
            .biometric_unlock_enabled
            .unwrap_or(current.biometric_unlock_enabled),
    }
}
