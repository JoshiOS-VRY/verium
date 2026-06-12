//! Register iOS device tokens with the Vericonomy push API (Electrum watchers + APNs).

use serde::Serialize;

use crate::chain::electrum::scripthash::scripthash_from_script_hex;
use crate::coin_profile::CoinId;
use crate::error::{AppError, AppResult};
use crate::prefs;
use crate::wallet::keystore;

const DEFAULT_PUSH_API_URL: &str = "https://push.vericonomy.com";
const IOS_BUNDLE_ID: &str = "com.vericonomy.wallet.ios";

#[derive(Debug, Serialize)]
struct RegisterBody {
    device_token: String,
    platform: String,
    bundle_id: String,
    notify_vrm: bool,
    notify_vrc: bool,
    verium_scripthashes: Vec<String>,
    vericoin_scripthashes: Vec<String>,
}

#[derive(Debug, Serialize)]
struct UnregisterBody {
    device_token: String,
}

/// Whether the wallet can call the push registration API (compile-time or runtime secret).
pub fn push_api_secret_configured() -> bool {
    push_api_secret().is_some()
}

fn push_api_base() -> Option<String> {
    if let Ok(s) = std::env::var("VERICONOMY_PUSH_API_URL") {
        if !s.trim().is_empty() {
            return Some(s.trim().to_string());
        }
    }
    let compiled = option_env!("VERICONOMY_PUSH_API_URL").unwrap_or("");
    if !compiled.is_empty() {
        return Some(compiled.to_string());
    }
    Some(DEFAULT_PUSH_API_URL.to_string())
}

fn push_api_secret() -> Option<String> {
    if let Ok(s) = std::env::var("VERICONOMY_PUSH_API_SECRET") {
        if !s.trim().is_empty() {
            return Some(s.trim().to_string());
        }
    }
    let compiled = option_env!("VERICONOMY_PUSH_API_SECRET").unwrap_or("");
    if !compiled.is_empty() {
        return Some(compiled.to_string());
    }
    None
}

fn scripthashes_for_coin(coin: CoinId) -> AppResult<Vec<String>> {
    if !keystore::wallet_exists(coin)? || !keystore::is_unlocked(coin)? {
        return Ok(Vec::new());
    }
    let scripts = keystore::funded_script_hexes(coin)?;
    let mut out = Vec::with_capacity(scripts.len());
    for script_hex in scripts {
        out.push(scripthash_from_script_hex(&script_hex)?);
    }
    out.sort();
    out.dedup();
    Ok(out)
}

async fn build_register_body(device_token: &str) -> AppResult<RegisterBody> {
    let prefs = prefs::load().await.unwrap_or_default();
    let notify_vrm =
        prefs.notify_on_vrm_received && prefs::coin_enabled(&prefs, CoinId::Verium);
    let notify_vrc =
        prefs.notify_on_vrc_received && prefs::coin_enabled(&prefs, CoinId::Vericoin);

    let verium_scripthashes = if notify_vrm {
        scripthashes_for_coin(CoinId::Verium)?
    } else {
        Vec::new()
    };
    let vericoin_scripthashes = if notify_vrc {
        scripthashes_for_coin(CoinId::Vericoin)?
    } else {
        Vec::new()
    };

    Ok(RegisterBody {
        device_token: device_token.trim().to_string(),
        platform: "ios".to_string(),
        bundle_id: IOS_BUNDLE_ID.to_string(),
        notify_vrm,
        notify_vrc,
        verium_scripthashes,
        vericoin_scripthashes,
    })
}

async fn post_json(path: &str, body: &impl Serialize) -> AppResult<()> {
    let base = push_api_base().ok_or_else(|| {
        AppError::other("VERICONOMY_PUSH_API_URL not configured")
    })?;
    let secret = push_api_secret().ok_or_else(|| {
        AppError::other("VERICONOMY_PUSH_API_SECRET not configured")
    })?;

    let url = format!(
        "{}/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/')
    );
    let client = crate::http_shared::shared_http_client(
        std::time::Duration::from_secs(30),
        "vericonomy-wallet-ios/1.0",
    )?;
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", secret.trim()))
        .header("Content-Type", "application/json")
        .json(body)
        .send()
        .await
        .map_err(|e| AppError::other(format!("push API request failed: {e}")))?;

    if resp.status().is_success() {
        tracing::debug!("push API {path} ok");
        Ok(())
    } else {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        tracing::warn!("push API {path} failed: {status} {text}");
        Err(AppError::other(format!("push API {path}: {status}")))
    }
}

/// Upsert device + scripthash subscriptions on the push service.
pub async fn register_device(device_token: &str) -> AppResult<()> {
    if device_token.trim().is_empty() {
        return Ok(());
    }
    if push_api_secret().is_none() {
        tracing::debug!("push registration skipped: VERICONOMY_PUSH_API_SECRET unset");
        return Ok(());
    }
    let body = build_register_body(device_token).await?;
    post_json("/v1/devices/register", &body).await
}

/// Same payload as register; updates `last_seen_at` on the server.
pub async fn heartbeat_device(device_token: &str) -> AppResult<()> {
    if device_token.trim().is_empty() {
        return Ok(());
    }
    if push_api_secret().is_none() {
        return Ok(());
    }
    let body = build_register_body(device_token).await?;
    post_json("/v1/devices/heartbeat", &body).await
}

/// Remove device and all subscriptions (wallet lock / notify disabled).
pub async fn unregister_device(device_token: &str) -> AppResult<()> {
    if device_token.trim().is_empty() {
        return Ok(());
    }
    if push_api_secret().is_none() {
        return Ok(());
    }
    let body = UnregisterBody {
        device_token: device_token.trim().to_string(),
    };
    post_json("/v1/devices/unregister", &body).await
}
