//! Mobile wallet passphrase storage for Face ID / Touch ID unlock flows.
//!
//! The OS biometric prompt is handled by `tauri-plugin-biometric` on the frontend.
//! After a successful prompt, this module reads the stored passphrase and unlocks
//! the light wallet. Passphrases are sealed with the same encrypted blob store
//! as wallet prefs and keys (reliable on iOS; raw keychain entries were flaky).

use crate::coin_profile::CoinId;
use crate::error::{AppError, AppResult};
use crate::secret_store;

const LABEL_PREFIX: &str = "biometric-unlock-passphrase";
const LEGACY_ACCOUNT_PREFIX: &str = "biometric-wallet-passphrase";

fn label_for_coin(coin: CoinId) -> String {
    format!("{LABEL_PREFIX}:{}", coin.as_str())
}

fn legacy_keyring_entry(coin: CoinId) -> keyring::Entry {
    let account = format!("{LEGACY_ACCOUNT_PREFIX}:{}", coin.as_str());
    let service = if cfg!(target_os = "ios") {
        "com.vericonomy.wallet.ios"
    } else if cfg!(target_os = "android") {
        "com.vericonomy.wallet.android"
    } else {
        "com.vericonomy.wallet.desktop"
    };
    keyring::Entry::new(service, &account).expect("keyring entry")
}

fn migrate_legacy_keyring(coin: CoinId) -> AppResult<()> {
    match legacy_keyring_entry(coin).get_password() {
        Ok(passphrase) => {
            store_passphrase(coin, &passphrase)?;
            legacy_keyring_entry(coin).delete_credential().ok();
            tracing::info!(
                "biometric unlock: migrated legacy keychain entry for {}",
                coin.as_str()
            );
        }
        Err(keyring::Error::NoEntry) => {}
        Err(e) => {
            tracing::debug!(
                "biometric unlock: legacy keychain read skipped for {}: {e}",
                coin.as_str()
            );
        }
    }
    Ok(())
}

pub fn is_configured(coin: CoinId) -> AppResult<bool> {
    let label = label_for_coin(coin);
    if secret_store::blob_readable(&label) {
        return Ok(true);
    }
    migrate_legacy_keyring(coin)?;
    Ok(secret_store::blob_readable(&label))
}

pub fn store_passphrase(coin: CoinId, passphrase: &str) -> AppResult<()> {
    let label = label_for_coin(coin);
    secret_store::seal(&label, passphrase.as_bytes())?;
    legacy_keyring_entry(coin).delete_credential().ok();
    Ok(())
}

pub fn load_passphrase(coin: CoinId) -> AppResult<Option<String>> {
    let label = label_for_coin(coin);
    if let Some(bytes) = secret_store::open(&label)? {
        return Ok(Some(
            String::from_utf8(bytes.to_vec()).map_err(|e| AppError::other(format!(
                "biometric unlock passphrase is not valid UTF-8: {e}"
            )))?,
        ));
    }
    migrate_legacy_keyring(coin)?;
    if let Some(bytes) = secret_store::open(&label)? {
        return Ok(Some(
            String::from_utf8(bytes.to_vec()).map_err(|e| AppError::other(format!(
                "biometric unlock passphrase is not valid UTF-8: {e}"
            )))?,
        ));
    }
    Ok(None)
}

pub fn clear_passphrase(coin: CoinId) {
    let label = label_for_coin(coin);
    if let Err(e) = secret_store::delete(&label) {
        tracing::debug!(
            "biometric unlock: clear skipped for {}: {e}",
            coin.as_str()
        );
    }
    if let Err(e) = legacy_keyring_entry(coin).delete_credential() {
        tracing::debug!(
            "biometric unlock: legacy clear skipped for {}: {e}",
            coin.as_str()
        );
    }
}
