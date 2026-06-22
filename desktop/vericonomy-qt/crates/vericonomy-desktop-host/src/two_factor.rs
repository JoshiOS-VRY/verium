//! TOTP two-factor authentication (Tauri-compatible).

use serde::{Deserialize, Serialize};
use totp_rs::{Algorithm as TotpAlgorithm, Secret, TOTP};

use crate::error::{HostError, HostResult};

const STORE_PATH: &str = "two_factor.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoFactorConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub secret_base32: Option<String>,
    #[serde(default)]
    pub recovery_code_hashes: Vec<String>,
    #[serde(default)]
    pub used_recovery_hashes: Vec<String>,
    #[serde(default = "default_gated_actions")]
    pub gated_actions: Vec<String>,
    #[serde(default)]
    pub send_threshold_vrm: Option<f64>,
    #[serde(default)]
    pub send_threshold_vrc: Option<f64>,
    #[serde(default)]
    pub disabled_at: Option<i64>,
}

fn default_gated_actions() -> Vec<String> {
    vec![
        "send".into(),
        "change_passphrase".into(),
        "show_recovery_phrase".into(),
        "dump_privkey".into(),
        "restore_wallet".into(),
        "edit_conf".into(),
        "disable_2fa".into(),
    ]
}

impl Default for TwoFactorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            secret_base32: None,
            recovery_code_hashes: Vec::new(),
            used_recovery_hashes: Vec::new(),
            gated_actions: default_gated_actions(),
            send_threshold_vrm: Some(0.0),
            send_threshold_vrc: Some(0.0),
            disabled_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TwoFactorEnrollment {
    pub secret_base32: String,
    pub otpauth_uri: String,
    pub recovery_codes: Vec<String>,
}

fn config_path() -> std::path::PathBuf {
    crate::config::app_config_base().join(STORE_PATH)
}

pub fn load() -> HostResult<TwoFactorConfig> {
    let path = config_path();
    if !path.exists() {
        return Ok(TwoFactorConfig::default());
    }
    let raw = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw).unwrap_or_default())
}

pub fn save(config: &TwoFactorConfig) -> HostResult<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(config)?)?;
    Ok(())
}

fn hash_recovery_code(code: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    hex::encode(hasher.finalize())
}

fn build_totp(secret_b32: &str) -> HostResult<TOTP> {
    let secret = Secret::Encoded(secret_b32.to_string())
        .to_bytes()
        .map_err(|e| HostError::other(format!("invalid TOTP secret: {e}")))?;
    TOTP::new(
        TotpAlgorithm::SHA1,
        6,
        1,
        30,
        secret,
        Some("Vericonomy Wallet".to_string()),
        "wallet".to_string(),
    )
    .map_err(|e| HostError::other(format!("TOTP init failed: {e}")))
}

pub fn status() -> HostResult<TwoFactorConfig> {
    load()
}

pub fn start_enrollment() -> HostResult<TwoFactorEnrollment> {
    let secret = Secret::generate_secret();
    let secret_b32 = secret.to_encoded().to_string();
    let totp = build_totp(&secret_b32)?;
    let otpauth_uri = totp.get_url();
    let recovery_codes: Vec<String> = (0..8)
        .map(|i| format!("{:04}-{:04}", i * 1111 + 1234, i * 2222 + 5678))
        .collect();
    Ok(TwoFactorEnrollment {
        secret_base32: secret_b32,
        otpauth_uri,
        recovery_codes,
    })
}

pub fn confirm_enrollment(code: &str, enrollment_secret: Option<&str>) -> HostResult<TwoFactorConfig> {
    let secret = enrollment_secret.ok_or_else(|| HostError::other("missing enrollment secret"))?;
    let totp = build_totp(secret)?;
    if !totp.check_current(code).unwrap_or(false) {
        return Err(HostError::other("invalid verification code"));
    }
    let mut config = load()?;
    config.enabled = true;
    config.secret_base32 = Some(secret.to_string());
    config.disabled_at = None;
    save(&config)?;
    Ok(config)
}

pub fn verify(code: &str) -> HostResult<bool> {
    let config = load()?;
    if !config.enabled {
        return Ok(true);
    }
    let secret = config
        .secret_base32
        .as_deref()
        .ok_or_else(|| HostError::other("2FA enabled but secret missing"))?;
    let totp = build_totp(secret)?;
    Ok(totp.check_current(code).unwrap_or(false))
}

pub fn disable(code: &str) -> HostResult<()> {
    if !verify(code)? {
        return Err(HostError::other("invalid verification code"));
    }
    let mut config = load()?;
    config.enabled = false;
    config.secret_base32 = None;
    config.disabled_at = Some(now_ts());
    save(&config)
}

pub fn is_action_gated(action: &str, amount: Option<f64>, coin: &str) -> HostResult<bool> {
    let config = load()?;
    if !config.enabled {
        return Ok(false);
    }
    if !config.gated_actions.iter().any(|a| a == action) {
        return Ok(false);
    }
    if action == "send" {
        let threshold = match coin {
            "vericoin" => config.send_threshold_vrc.unwrap_or(0.0),
            _ => config.send_threshold_vrm.unwrap_or(0.0),
        };
        if let Some(amt) = amount {
            return Ok(amt >= threshold);
        }
    }
    Ok(true)
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
