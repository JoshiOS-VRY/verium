//! Tauri commands for light wallet mode, settings, and server validation.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::chain::electrum::conformance::{run_all_default_servers, run_conformance, ConformanceResult};
use crate::chain::electrum::multi_server::{verify_tip_consistency, TipVerifyResult};
use crate::coin_profile::CoinId;
use crate::coin_profile::parse_coin_id;
use crate::error::AppResult;
use crate::prefs::{self, UserPreferences};
use crate::recovery;
use crate::state::AppState;
use crate::wallet::keystore;
use crate::wallet::mode::WalletMode;
use crate::wallet::service;

async fn ensure_light_wallet_mode_active() -> AppResult<()> {
    let mut prefs = prefs::load().await?;
    if prefs.wallet_mode.is_light() {
        return Ok(());
    }
    prefs.wallet_mode = WalletMode::Light;
    prefs::save(&prefs).await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletModeStatus {
    pub mode: String,
    pub light_wallet_enabled: bool,
    pub light_wallet_exists: bool,
    pub electrum_servers: Vec<String>,
}

#[tauri::command]
pub async fn wallet_mode_get() -> AppResult<WalletModeStatus> {
    let prefs = prefs::load().await?;
    let coin = prefs.active_coin.parse().unwrap_or(CoinId::Verium);
    let network = crate::features::effective_network_mode(prefs.network_mode);
    Ok(WalletModeStatus {
        mode: prefs.wallet_mode.as_str().to_string(),
        light_wallet_enabled: crate::features::light_wallet_enabled(),
        light_wallet_exists: keystore::wallet_exists(coin).unwrap_or(false),
        electrum_servers: prefs
            .electrum_servers_by_coin
            .as_ref()
            .and_then(|m| m.get(coin.as_str()).cloned())
            .unwrap_or_else(|| coin.default_electrum_servers(network)),
    })
}

#[tauri::command]
pub async fn wallet_mode_set(mode: String) -> AppResult<()> {
    let mut prefs = prefs::load().await?;
    prefs.wallet_mode = WalletMode::from_str_lossy(&mode);
    prefs::save(&prefs).await
}

#[tauri::command]
pub async fn electrum_servers_get(coin: String) -> AppResult<Vec<String>> {
    let coin = parse_coin_id(&coin)?;
    let prefs = prefs::load().await?;
    let network = crate::features::effective_network_mode(prefs.network_mode);
    Ok(prefs
        .electrum_servers_by_coin
        .as_ref()
        .and_then(|m| m.get(coin.as_str()).cloned())
        .unwrap_or_else(|| coin.default_electrum_servers(network)))
}

#[tauri::command]
pub async fn electrum_servers_set(coin: String, servers: Vec<String>) -> AppResult<()> {
    let coin = parse_coin_id(&coin)?;
    let mut prefs = prefs::load().await?;
    let mut map = prefs.electrum_servers_by_coin.unwrap_or_default();
    map.insert(
        coin.as_str().to_string(),
        servers
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
    );
    prefs.electrum_servers_by_coin = Some(map);
    prefs::save(&prefs).await
}

#[tauri::command]
pub async fn electrum_test_connection(coin: String, server: Option<String>) -> AppResult<ConformanceResult> {
    let coin = parse_coin_id(&coin)?;
    let prefs = prefs::load().await?;
    let network = crate::features::effective_network_mode(prefs.network_mode);
    let uri = server.unwrap_or_else(|| {
        coin.default_electrum_servers(network)
            .first()
            .cloned()
            .unwrap_or_default()
    });
    run_conformance(coin, &uri).await
}

#[tauri::command]
pub async fn electrum_validate_defaults(coin: String) -> AppResult<Vec<ConformanceResult>> {
    let coin = parse_coin_id(&coin)?;
    run_all_default_servers(coin).await
}

#[tauri::command]
pub async fn light_wallet_create(
    coin: String,
    mnemonic: String,
    passphrase: String,
    label: Option<String>,
) -> AppResult<()> {
    let coin = parse_coin_id(&coin)?;
    let trimmed = crate::wallet::hd::normalize_hd_master_secret(&mnemonic);
    if crate::wallet::hd::is_hd_master_secret(coin, &trimmed) {
        crate::wallet::hd::parse_root_xpriv(coin, &trimmed)?;
    } else if !recovery::validate_mnemonic(&trimmed)? {
        return Err(crate::error::AppError::other(
            "invalid recovery phrase — enter a 24-word BIP39 phrase or the HD master key from Security → Export",
        ));
    }
    keystore::create_wallet(coin, &trimmed, &passphrase, label.as_deref())?;
    ensure_light_wallet_mode_active().await
}

#[tauri::command]
pub async fn light_wallet_import(
    state: State<'_, AppState>,
    coin: String,
    mnemonic: String,
    passphrase: String,
    label: Option<String>,
    totp_code: Option<String>,
) -> AppResult<()> {
    crate::security_policy::require_gated_action("restore_wallet", totp_code.as_deref())?;
    let coin = parse_coin_id(&coin)?;
    let trimmed = crate::wallet::hd::normalize_hd_master_secret(&mnemonic);
    if crate::wallet::hd::is_hd_master_secret(coin, &trimmed) {
        crate::wallet::hd::parse_root_xpriv(coin, &trimmed)?;
    } else if !recovery::validate_mnemonic(&trimmed)? {
        return Err(crate::error::AppError::other(
            "invalid recovery phrase — enter a 24-word BIP39 phrase or the HD master key from Security → Export",
        ));
    }
    keystore::import_wallet(coin, &trimmed, &passphrase, label.as_deref())?;
    if !keystore::wallet_exists(coin)? {
        return Err(crate::error::AppError::other(
            "import did not persist — could not write the light wallet file. Check disk space and retry.",
        ));
    }
    ensure_light_wallet_mode_active().await?;
    keystore::mark_address_scan_incomplete(coin)?;
    let app = state.inner().clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = crate::wallet::sync::sync_light_wallet(&app, coin).await {
            tracing::error!("light wallet import sync failed for {}: {e}", coin.as_str());
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn light_wallet_unlock(
    state: State<'_, AppState>,
    coin: String,
    passphrase: String,
    seconds: Option<u32>,
) -> AppResult<()> {
    let coin = parse_coin_id(&coin)?;
    keystore::unlock_wallet(coin, &passphrase, seconds.unwrap_or(4 * 60 * 60))?;
    ensure_light_wallet_mode_active().await?;
    let app = state.inner().clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = crate::wallet::sync::sync_light_wallet(&app, coin).await {
            tracing::error!("light wallet unlock sync failed for {}: {e}", coin.as_str());
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn light_wallet_rescan(
    state: State<'_, AppState>,
    coin: String,
) -> AppResult<()> {
    let coin = parse_coin_id(&coin)?;
    if !keystore::is_unlocked(coin)? {
        return Err(crate::error::AppError::other(
            "unlock your light wallet before rescanning addresses",
        ));
    }
    keystore::mark_address_scan_incomplete(coin)?;
    crate::wallet::sync::sync_light_wallet(&state, coin).await
}

#[tauri::command]
pub async fn light_wallet_lock(coin: String) -> AppResult<()> {
    let coin = parse_coin_id(&coin)?;
    keystore::lock_wallet(coin)
}

#[tauri::command]
pub async fn light_wallet_exists(coin: String) -> AppResult<bool> {
    let coin = parse_coin_id(&coin)?;
    keystore::wallet_exists(coin)
}

#[tauri::command]
pub async fn light_server_status(
    state: State<'_, AppState>,
    coin: String,
) -> AppResult<Option<crate::chain::types::LightServerStatus>> {
    let coin = parse_coin_id(&coin)?;
    service::light_server_status(&state, coin).await
}

#[tauri::command]
pub async fn electrum_cross_verify_tip(coin: String) -> AppResult<TipVerifyResult> {
    let coin = parse_coin_id(&coin)?;
    let prefs = prefs::load().await?;
    let network = crate::features::effective_network_mode(prefs.network_mode);
    let servers = prefs
        .electrum_servers_by_coin
        .as_ref()
        .and_then(|m| m.get(coin.as_str()).cloned())
        .unwrap_or_else(|| coin.default_electrum_servers(network));
    verify_tip_consistency(coin, &servers, 1).await
}
