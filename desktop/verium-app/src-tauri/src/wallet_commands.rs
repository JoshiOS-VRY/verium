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

/// Mark one coin as light mode after a light wallet is created/imported/unlocked.
/// Per-coin so the other chain can keep running a full node.
async fn ensure_light_wallet_mode_active(coin: CoinId) -> AppResult<()> {
    let mut prefs = prefs::load().await?;
    if prefs::wallet_mode_for(&prefs, coin).is_light() {
        return Ok(());
    }
    prefs::set_wallet_mode_for(&mut prefs, coin, WalletMode::Light);
    prefs::save(&prefs).await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletModeStatus {
    pub mode: String,
    pub light_wallet_enabled: bool,
    pub light_wallet_exists: bool,
    pub electrum_servers: Vec<String>,
    /// True on iOS/Android — app runs Electrum light wallet only (no local node).
    pub mobile_only: bool,
}

fn wallet_mode_status_for(prefs: &UserPreferences, coin: CoinId) -> WalletModeStatus {
    let network = crate::features::effective_network_mode(prefs.network_mode);
    WalletModeStatus {
        mode: prefs::wallet_mode_for(prefs, coin).as_str().to_string(),
        light_wallet_enabled: crate::features::light_wallet_enabled(),
        light_wallet_exists: keystore::light_wallet_on_disk(coin),
        electrum_servers: prefs
            .electrum_servers_by_coin
            .as_ref()
            .and_then(|m| m.get(coin.as_str()).cloned())
            .unwrap_or_else(|| coin.default_electrum_servers(network)),
        mobile_only: crate::features::is_light_wallet(),
    }
}

#[tauri::command]
pub async fn wallet_mode_get() -> AppResult<WalletModeStatus> {
    let prefs = prefs::load().await?;
    let coin = prefs.active_coin.parse().unwrap_or(CoinId::Verium);
    Ok(wallet_mode_status_for(&prefs, coin))
}

/// Per-coin variant of `wallet_mode_get`.
#[tauri::command]
pub async fn wallet_mode_get_for_coin(coin: String) -> AppResult<WalletModeStatus> {
    let coin = parse_coin_id(&coin)?;
    let prefs = prefs::load().await?;
    Ok(wallet_mode_status_for(&prefs, coin))
}

#[tauri::command]
pub async fn wallet_mode_set(mode: String) -> AppResult<()> {
    if crate::features::is_light_wallet() && mode != "light" {
        return Err(crate::error::AppError::other(
            "Mobile builds only support light wallet (Electrum) mode.",
        ));
    }
    let mut prefs = prefs::load().await?;
    // App-wide default still set for backward compatibility; per-coin overrides
    // win when present (see prefs::wallet_mode_for).
    prefs.wallet_mode = WalletMode::from_str_lossy(&mode);
    let _ = prefs::reconcile_setup_flags_with_keystore(&mut prefs)?;
    prefs::save(&prefs).await
}

/// Set the wallet mode for a single coin without touching the other chain.
#[tauri::command]
pub async fn wallet_mode_set_for_coin(coin: String, mode: String) -> AppResult<()> {
    if crate::features::is_light_wallet() && mode != "light" {
        return Err(crate::error::AppError::other(
            "Mobile builds only support light wallet (Electrum) mode.",
        ));
    }
    let coin = parse_coin_id(&coin)?;
    let mut prefs = prefs::load().await?;
    prefs::set_wallet_mode_for(&mut prefs, coin, WalletMode::from_str_lossy(&mode));
    let _ = prefs::reconcile_setup_flags_with_keystore(&mut prefs)?;
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
    state: State<'_, AppState>,
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
    ensure_light_wallet_mode_active(coin).await?;
    let mut prefs = prefs::load().await?;
    prefs::reconcile_setup_flags_with_keystore(&mut prefs)?;
    prefs::save(&prefs).await?;
    prefs::invalidate_prefs_cache();
    keystore::mark_address_scan_incomplete(coin)?;
    let app = state.inner().clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = crate::wallet::sync::sync_light_wallet(&app, coin).await {
            tracing::error!("light wallet create sync failed for {}: {e}", coin.as_str());
        }
    });
    Ok(())
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
    ensure_light_wallet_mode_active(coin).await?;
    let mut prefs = prefs::load().await?;
    prefs::reconcile_setup_flags_with_keystore(&mut prefs)?;
    prefs::save(&prefs).await?;
    prefs::invalidate_prefs_cache();
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
    ensure_light_wallet_mode_active(coin).await?;
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
    // Release the pooled Electrum connection for this coin when locking.
    crate::wallet::backend::drop_pooled_electrum_client(coin);
    keystore::lock_wallet(coin)
}

#[tauri::command]
pub async fn light_wallet_exists(coin: String) -> AppResult<bool> {
    let coin = parse_coin_id(&coin)?;
    Ok(keystore::light_wallet_on_disk(coin))
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
