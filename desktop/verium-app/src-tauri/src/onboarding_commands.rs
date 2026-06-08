//! Unified per-coin wallet profile + onboarding checkpoint commands.
//!
//! `WalletProfile` is the single readiness authority the frontend consults to
//! decide whether a coin still needs setup. It replaces the previously-split
//! `isCoinWalletReady` (mode-specific keys) and `isCoinSetupComplete`
//! (prefs flag OR any stored wallet) logic, which could disagree depending on
//! the entry point. Readiness here is defined as: keys exist on disk for the
//! coin's effective wallet mode. Unlock state is a runtime concern owned by the
//! daemon/keystore, not onboarding.

use serde::Serialize;
use tauri::State;

use crate::coin_profile::parse_coin_id;
use crate::config::{
    resolve_legacy_datadir_outside_cfg, resolve_legacy_wallet_outside_cfg, wallet_dat_exists,
};
use crate::error::AppResult;
use crate::onboarding::{OnboardingCheckpoint, OnboardingIntent, OnboardingPhase};
use crate::prefs;
use crate::state::AppState;
use crate::wallet::keystore;
use crate::wallet::mode::WalletMode;

#[derive(Debug, Clone, Serialize)]
pub struct KeysPresent {
    /// `wallet.dat` exists in the configured datadir for this coin.
    pub full_node: bool,
    /// An encrypted light keystore exists for this coin.
    pub light: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LegacyInfo {
    /// A legacy Qt `wallet.dat` was found outside the configured datadir.
    pub detected: bool,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HdInfo {
    /// An encrypted BIP39 recovery-phrase backup is on file for this coin.
    /// (Full HD status requires a running daemon and is resolved separately
    /// via `recovery_wallet_is_hd`; the profile stays daemon-independent.)
    pub has_mnemonic_backup: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WalletProfile {
    pub coin: String,
    /// Effective per-coin wallet mode (`full_node` | `light`).
    pub mode: String,
    pub keys_present: KeysPresent,
    pub legacy: LegacyInfo,
    pub hd: HdInfo,
    pub onboarding: OnboardingCheckpoint,
    /// Classified entry intent for the wizard (`fresh_install`, ...).
    pub intent: String,
    /// True when keys exist for the active mode — the only gate for dashboard
    /// access. Does not depend on `setup_completed` flags.
    pub ready: bool,
}

fn classify_intent(
    mode: WalletMode,
    keys: &KeysPresent,
    legacy: &LegacyInfo,
) -> OnboardingIntent {
    if mode.is_light() {
        if keys.light {
            OnboardingIntent::LightContinue
        } else if keys.full_node || legacy.detected {
            OnboardingIntent::CrossModeMigration
        } else {
            OnboardingIntent::FreshInstall
        }
    } else if keys.full_node {
        OnboardingIntent::ExistingUnlock
    } else if legacy.detected {
        OnboardingIntent::LegacyUpgrade
    } else if keys.light {
        OnboardingIntent::CrossModeMigration
    } else {
        OnboardingIntent::FreshInstall
    }
}

#[tauri::command]
pub async fn wallet_profile(
    state: State<'_, AppState>,
    coin: String,
) -> AppResult<WalletProfile> {
    let coin = parse_coin_id(&coin)?;
    let prefs = prefs::load().await?;
    let mode = prefs::wallet_mode_for(&prefs, coin);
    let cfg = state.config_fresh(coin).await?;

    let keys = KeysPresent {
        full_node: wallet_dat_exists(coin, &cfg),
        light: keystore::wallet_exists(coin).unwrap_or(false),
    };
    let legacy_path = resolve_legacy_wallet_outside_cfg(coin, &cfg);
    let legacy = LegacyInfo {
        detected: legacy_path.is_some(),
        path: legacy_path.map(|p| p.display().to_string()),
    };
    let hd = HdInfo {
        has_mnemonic_backup: crate::mnemonic_backup::exists(coin),
    };

    let ready = match mode {
        WalletMode::Light => keys.light,
        WalletMode::FullNode => keys.full_node,
    };
    let intent = classify_intent(mode, &keys, &legacy);

    Ok(WalletProfile {
        coin: coin.as_str().to_string(),
        mode: mode.as_str().to_string(),
        keys_present: keys,
        legacy,
        hd,
        onboarding: prefs::onboarding_for(&prefs, coin),
        intent: intent.as_str().to_string(),
        ready,
    })
}

#[tauri::command]
pub async fn onboarding_checkpoint_get(coin: String) -> AppResult<OnboardingCheckpoint> {
    let coin = parse_coin_id(&coin)?;
    let prefs = prefs::load().await?;
    Ok(prefs::onboarding_for(&prefs, coin))
}

#[tauri::command]
pub async fn onboarding_checkpoint_set(
    coin: String,
    checkpoint: OnboardingCheckpoint,
) -> AppResult<()> {
    let coin = parse_coin_id(&coin)?;
    let mut prefs = prefs::load().await?;
    prefs::set_onboarding_for(&mut prefs, coin, checkpoint);
    prefs::save(&prefs).await
}

/// Mark a coin's onboarding finished. Also sets the legacy `setup_completed*`
/// flags so older code paths and prefs reconciliation stay consistent.
#[tauri::command]
pub async fn onboarding_mark_complete(coin: String) -> AppResult<()> {
    let coin = parse_coin_id(&coin)?;
    let mut prefs = prefs::load().await?;

    let mut checkpoint = prefs::onboarding_for(&prefs, coin);
    checkpoint.phase = OnboardingPhase::Complete;
    checkpoint.step = Some("done".to_string());
    prefs::set_onboarding_for(&mut prefs, coin, checkpoint);

    let mut setup = prefs.setup_completed_by_coin.clone().unwrap_or_default();
    setup.insert(coin.as_str().to_string(), true);
    prefs.setup_completed_by_coin = Some(setup);
    if coin == crate::coin_profile::CoinId::Verium {
        prefs.setup_completed = true;
    }

    prefs::save(&prefs).await
}

/// Return the legacy datadir the user can adopt for this coin (the folder
/// containing the detected legacy `wallet.dat`), or `None` if there is nothing
/// to adopt. The frontend offers this as a one-click "use my existing data
/// folder" action, then calls `set_daemon_config` + `restart_daemon`.
#[tauri::command]
pub async fn legacy_datadir_candidate(
    state: State<'_, AppState>,
    coin: String,
) -> AppResult<Option<String>> {
    let coin = parse_coin_id(&coin)?;
    let cfg = state.config_fresh(coin).await?;
    Ok(resolve_legacy_datadir_outside_cfg(coin, &cfg).map(|p| p.display().to_string()))
}

/// Request a one-shot `-upgradewallet` and restart the daemon. This upgrades a
/// pre-HD `wallet.dat` so a recovery phrase can be applied via `sethdseed`.
/// Used by the legacy upgrade wizard before the HD/recovery step.
#[tauri::command]
pub async fn legacy_request_hd_upgrade(
    state: State<'_, AppState>,
    coin: String,
) -> AppResult<()> {
    let coin = parse_coin_id(&coin)?;
    crate::daemon::request_wallet_upgrade_once(coin);
    crate::commands::restart_daemon_full_cycle(state.inner(), coin).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(full: bool, light: bool) -> KeysPresent {
        KeysPresent {
            full_node: full,
            light,
        }
    }

    fn legacy(detected: bool) -> LegacyInfo {
        LegacyInfo {
            detected,
            path: detected.then(|| "/old/wallet.dat".to_string()),
        }
    }

    #[test]
    fn fresh_install_when_nothing_present() {
        let intent = classify_intent(WalletMode::FullNode, &keys(false, false), &legacy(false));
        assert_eq!(intent, OnboardingIntent::FreshInstall);
    }

    #[test]
    fn legacy_upgrade_when_only_legacy_wallet_outside_cfg() {
        let intent = classify_intent(WalletMode::FullNode, &keys(false, false), &legacy(true));
        assert_eq!(intent, OnboardingIntent::LegacyUpgrade);
    }

    #[test]
    fn existing_unlock_when_wallet_dat_in_cfg() {
        let intent = classify_intent(WalletMode::FullNode, &keys(true, false), &legacy(true));
        assert_eq!(intent, OnboardingIntent::ExistingUnlock);
    }

    #[test]
    fn light_continue_when_keystore_present() {
        let intent = classify_intent(WalletMode::Light, &keys(false, true), &legacy(false));
        assert_eq!(intent, OnboardingIntent::LightContinue);
    }

    #[test]
    fn cross_mode_when_other_mode_has_keys() {
        // Light mode selected, but only a full-node wallet exists.
        let light_intent =
            classify_intent(WalletMode::Light, &keys(true, false), &legacy(false));
        assert_eq!(light_intent, OnboardingIntent::CrossModeMigration);
        // Full mode selected, but only a light keystore exists.
        let full_intent =
            classify_intent(WalletMode::FullNode, &keys(false, true), &legacy(false));
        assert_eq!(full_intent, OnboardingIntent::CrossModeMigration);
    }
}
