//! Anti-phishing spending controls (Tauri-compatible JSON on disk).

use serde::{Deserialize, Serialize};

use crate::coin::CoinId;
use crate::commands::addressbook;
use crate::error::HostResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendingControlsConfig {
    #[serde(default)]
    pub daily_spend_cap_vrm: Option<f64>,
    #[serde(default)]
    pub daily_spend_cap_vrc: Option<f64>,
    #[serde(default)]
    pub allowlist_only: bool,
    #[serde(default)]
    pub require_first_send_confirmation: bool,
    #[serde(default)]
    pub clipboard_guard_enabled: bool,
    #[serde(default)]
    pub spent_today_vrm: f64,
    #[serde(default)]
    pub spent_today_vrc: f64,
    #[serde(default)]
    pub spend_day: Option<String>,
    #[serde(default)]
    pub sent_addresses: Vec<String>,
}

impl Default for SpendingControlsConfig {
    fn default() -> Self {
        Self {
            daily_spend_cap_vrm: None,
            daily_spend_cap_vrc: None,
            allowlist_only: false,
            require_first_send_confirmation: true,
            clipboard_guard_enabled: true,
            spent_today_vrm: 0.0,
            spent_today_vrc: 0.0,
            spend_day: None,
            sent_addresses: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SpendCheckResult {
    pub allowed: bool,
    pub reason: Option<String>,
    pub requires_extra_confirmation: bool,
    pub look_alike_warning: Option<String>,
}

fn config_path() -> std::path::PathBuf {
    crate::config::app_config_base().join("spending_controls.json")
}

pub fn load() -> HostResult<SpendingControlsConfig> {
    let path = config_path();
    if !path.exists() {
        return Ok(SpendingControlsConfig::default());
    }
    let raw = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw).unwrap_or_default())
}

pub fn save(config: &SpendingControlsConfig) -> HostResult<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(config)?)?;
    Ok(())
}

pub fn save_ui_settings(incoming: &SpendingControlsConfig) -> HostResult<()> {
    let mut config = load()?;
    config.daily_spend_cap_vrm = incoming.daily_spend_cap_vrm;
    config.daily_spend_cap_vrc = incoming.daily_spend_cap_vrc;
    config.allowlist_only = incoming.allowlist_only;
    config.require_first_send_confirmation = incoming.require_first_send_confirmation;
    config.clipboard_guard_enabled = incoming.clipboard_guard_enabled;
    save(&config)
}

fn today_str() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() / 86400)
        .unwrap_or(0);
    format!("day-{days}")
}

fn reset_daily_if_needed(config: &mut SpendingControlsConfig) {
    let today = today_str();
    if config.spend_day.as_deref() != Some(&today) {
        config.spent_today_vrm = 0.0;
        config.spent_today_vrc = 0.0;
        config.spend_day = Some(today);
    }
}

pub fn is_known_send_destination(coin: &str, address: &str, config: &SpendingControlsConfig) -> bool {
    let address = address.trim();
    if address.is_empty() {
        return false;
    }
    if config
        .sent_addresses
        .iter()
        .any(|a| a.eq_ignore_ascii_case(address))
    {
        return true;
    }
    if let Some(coin_id) = CoinId::parse(coin) {
        if let Ok(entries) = addressbook::list(coin_id) {
            return entries.iter().any(|entry| {
                entry.category.eq_ignore_ascii_case("send")
                    && entry.address.eq_ignore_ascii_case(address)
            });
        }
    }
    false
}

pub fn check_spend_allowed(
    amount: f64,
    coin: &str,
    address: &str,
    wallet_already_sent: bool,
) -> HostResult<SpendCheckResult> {
    let mut config = load()?;
    reset_daily_if_needed(&mut config);

    let cap = match coin {
        "vericoin" => config.daily_spend_cap_vrc,
        _ => config.daily_spend_cap_vrm,
    };
    let spent = match coin {
        "vericoin" => config.spent_today_vrc,
        _ => config.spent_today_vrm,
    };

    if let Some(cap_val) = cap {
        if spent + amount > cap_val {
            return Ok(SpendCheckResult {
                allowed: false,
                reason: Some(format!(
                    "Daily spend cap exceeded ({spent:.8} + {amount:.8} > {cap_val:.8})"
                )),
                requires_extra_confirmation: false,
                look_alike_warning: None,
            });
        }
    }

    let is_first_send = !is_known_send_destination(coin, address, &config) && !wallet_already_sent;
    let requires_extra = config.require_first_send_confirmation && is_first_send;

    Ok(SpendCheckResult {
        allowed: true,
        reason: None,
        requires_extra_confirmation: requires_extra,
        look_alike_warning: detect_look_alike(address, &known_addresses_for_lookalike(coin, &config)),
    })
}

fn known_addresses_for_lookalike(coin: &str, config: &SpendingControlsConfig) -> Vec<String> {
    let mut known = config.sent_addresses.clone();
    if let Some(coin_id) = CoinId::parse(coin) {
        if let Ok(entries) = addressbook::list(coin_id) {
            for entry in entries {
                if entry.category.eq_ignore_ascii_case("send") {
                    known.push(entry.address);
                }
            }
        }
    }
    known
}

pub fn record_spend(amount: f64, coin: &str, address: &str) -> HostResult<()> {
    let mut config = load()?;
    reset_daily_if_needed(&mut config);
    match coin {
        "vericoin" => config.spent_today_vrc += amount,
        _ => config.spent_today_vrm += amount,
    }
    let address = address.trim();
    if !address.is_empty()
        && !config
            .sent_addresses
            .iter()
            .any(|a| a.eq_ignore_ascii_case(address))
    {
        config.sent_addresses.push(address.to_string());
    }
    save(&config)
}

pub fn check_allowlist(address: &str, allowlist: &[String]) -> bool {
    let address = address.trim();
    allowlist
        .iter()
        .any(|a| a.eq_ignore_ascii_case(address))
}

pub fn detect_look_alike(address: &str, known: &[String]) -> Option<String> {
    let address = address.trim();
    if address.len() < 8 {
        return None;
    }
    let prefix = &address[..4.min(address.len())];
    let suffix = &address[address.len().saturating_sub(4)..];
    for known_addr in known {
        if known_addr.eq_ignore_ascii_case(address) {
            return None;
        }
        if known_addr.len() >= 8
            && known_addr.starts_with(prefix)
            && known_addr.ends_with(suffix)
            && !known_addr.eq_ignore_ascii_case(address)
        {
            return Some(format!(
                "This address looks similar to a saved address ({known_addr}). Double-check before sending."
            ));
        }
    }
    None
}

pub fn check_send_with_allowlist(
    coin: CoinId,
    address: &str,
    amount: f64,
    wallet_already_sent: bool,
) -> HostResult<SpendCheckResult> {
    let config = load()?;
    let coin_str = coin.as_str();
    if config.allowlist_only {
        let entries = addressbook::list(coin)?;
        let allowlist: Vec<String> = entries
            .iter()
            .filter(|e| e.category.eq_ignore_ascii_case("send"))
            .map(|e| e.address.clone())
            .collect();
        if !check_allowlist(address, &allowlist) {
            return Ok(SpendCheckResult {
                allowed: false,
                reason: Some("Address is not on your send allowlist.".into()),
                requires_extra_confirmation: false,
                look_alike_warning: None,
            });
        }
    }
    check_spend_allowed(amount, coin_str, address, wallet_already_sent)
}
