//! Central security policy enforcement for sensitive Tauri commands.
//!
//! All wallet operations that move funds, export secrets, or change security
//! configuration must call into this module so UI-only gates cannot be bypassed
//! via direct Tauri invoke.

use crate::coin_profile::CoinId;
use crate::error::{AppError, AppResult};
use crate::spending_controls;
use crate::state::AppState;
use crate::two_factor;
use serde_json::json;

/// Require TOTP verification when 2FA is enabled and the action is gated.
pub fn require_gated_action(action: &str, totp_code: Option<&str>) -> AppResult<()> {
    if !two_factor::is_action_gated(action, None, "")? {
        return Ok(());
    }
    let code = totp_code
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::other(format!("2FA code required for {action}")))?;
    if !two_factor::verify(code)? {
        return Err(AppError::other("Invalid 2FA code"));
    }
    Ok(())
}

/// Require TOTP when 2FA is enabled for sends, otherwise verify wallet passphrase.
pub async fn authorize_send(
    state: &AppState,
    coin: CoinId,
    totp_code: Option<&str>,
    wallet_passphrase: Option<&str>,
) -> AppResult<()> {
    if two_factor::is_action_gated("send", None, coin.as_str())? {
        require_gated_action("send", totp_code)?;
        let prefs = crate::prefs::load().await?;
        if crate::prefs::wallet_mode_for(&prefs, coin).is_light()
            && !crate::wallet::keystore::signing_session_active(coin)
        {
            return Err(AppError::other(
                "Unlock your light wallet before sending (Dashboard or Transactions).",
            ));
        }
        return Ok(());
    }
    verify_wallet_passphrase_for_send(state, coin, wallet_passphrase).await
}

async fn verify_wallet_passphrase_for_send(
    state: &AppState,
    coin: CoinId,
    wallet_passphrase: Option<&str>,
) -> AppResult<()> {
    let pass = wallet_passphrase
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| AppError::other("Wallet passphrase required to send"))?;

    let prefs = crate::prefs::load().await?;
    if crate::prefs::wallet_mode_for(&prefs, coin).is_light() {
        crate::wallet::keystore::verify_passphrase(coin, pass)?;
        return Ok(());
    }

    let client = state.rpc_client(coin).await?;
    let info: serde_json::Value = client.call("getwalletinfo", json!([])).await?;
    if info.get("unlocked_until").is_none() {
        return Ok(());
    }

    let _ = client.call_no_result("walletlock", json!([])).await;
    client
        .call_no_result("walletpassphrase", json!([pass, 120]))
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.to_ascii_lowercase().contains("passphrase") {
                AppError::other("Incorrect wallet passphrase")
            } else {
                e
            }
        })?;
    Ok(())
}

fn send_allowlist(coin: CoinId) -> AppResult<Vec<String>> {
    let entries = crate::addressbook::list_entries(coin)?;
    Ok(entries
        .into_iter()
        .filter(|e| e.category.eq_ignore_ascii_case("send"))
        .map(|e| e.address)
        .collect())
}

/// Enforce spending controls and allowlist before any outbound transfer.
/// Call [`authorize_send`] first so 2FA or wallet passphrase is verified.
pub fn require_send_allowed(
    coin: &str,
    amount: f64,
    address: &str,
    extra_confirmed: bool,
) -> AppResult<()> {
    let config = spending_controls::load().unwrap_or_default();
    let wallet_already_sent =
        spending_controls::is_known_send_destination(coin, address, &config);

    let check = spending_controls::check_spend_allowed(
        amount,
        coin,
        address,
        wallet_already_sent,
    )?;
    if !check.allowed {
        return Err(AppError::other(
            check
                .reason
                .unwrap_or_else(|| "Send blocked by spending controls".into()),
        ));
    }
    if check.requires_extra_confirmation && !extra_confirmed {
        return Err(AppError::other(
            "First send to this address requires explicit confirmation",
        ));
    }

    if let Ok(coin_id) = crate::coin_profile::parse_coin_id(coin) {
        let allowlist = send_allowlist(coin_id)?;
        if !spending_controls::check_allowlist(address, &allowlist) {
            return Err(AppError::other(
                "Allowlist mode: add this address to your address book (Send) first",
            ));
        }
    }

    Ok(())
}

/// Enforce policy for multi-output sends (coin control).
/// Call [`authorize_send`] first so 2FA or wallet passphrase is verified.
pub fn require_multi_send_allowed(
    coin: &str,
    outputs: &[(String, f64)],
    extra_confirmed: bool,
) -> AppResult<()> {
    let total: f64 = outputs.iter().map(|(_, a)| a).sum();

    for (address, amount) in outputs {
        let config = spending_controls::load().unwrap_or_default();
        let wallet_already_sent =
            spending_controls::is_known_send_destination(coin, address, &config);
        let check = spending_controls::check_spend_allowed(
            *amount,
            coin,
            address,
            wallet_already_sent,
        )?;
        if !check.allowed {
            return Err(AppError::other(
                check.reason.unwrap_or_else(|| {
                    format!("Send to {address} blocked by spending controls")
                }),
            ));
        }
        if check.requires_extra_confirmation && !extra_confirmed {
            return Err(AppError::other(format!(
                "First send to {address} requires explicit confirmation"
            )));
        }
        if let Ok(coin_id) = crate::coin_profile::parse_coin_id(coin) {
            let allowlist = send_allowlist(coin_id)?;
            if !spending_controls::check_allowlist(address, &allowlist) {
                return Err(AppError::other(format!(
                    "Allowlist mode: {address} is not in your address book"
                )));
            }
        }
    }

    // Daily cap applies to the transaction total.
    let config = spending_controls::load().unwrap_or_default();
    let cap = match coin {
        "vericoin" => config.daily_spend_cap_vrc,
        _ => config.daily_spend_cap_vrm,
    };
    let spent = match coin {
        "vericoin" => config.spent_today_vrc,
        _ => config.spent_today_vrm,
    };
    if let Some(cap_val) = cap {
        if spent + total > cap_val {
            return Err(AppError::other(format!(
                "Daily spend cap exceeded ({spent:.8} + {total:.8} > {cap_val:.8})"
            )));
        }
    }

    Ok(())
}

/// Record a successful spend for daily caps and first-send tracking.
pub fn record_send(coin: &str, amount: f64, address: &str) -> AppResult<()> {
    spending_controls::record_spend(amount, coin, address)
}

const REDACTED: &str = "***REDACTED***";

/// Redact RPC credentials from vericonomy.conf content before returning to UI.
pub fn redact_conf_content(content: &str) -> String {
    content
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            let lower = trimmed.to_ascii_lowercase();
            if lower.starts_with("rpcpassword=") || lower.starts_with("rpcuser=") {
                let key = trimmed.split('=').next().unwrap_or("");
                format!("{key}={REDACTED}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Restore redacted credential placeholders from the on-disk file before save.
pub fn restore_conf_secrets(incoming: &str, existing: &str) -> String {
    let secrets: std::collections::HashMap<String, String> = existing
        .lines()
        .filter_map(|line| {
            let lower = line.trim().to_ascii_lowercase();
            if lower.starts_with("rpcpassword=") || lower.starts_with("rpcuser=") {
                let mut parts = line.splitn(2, '=');
                let key = parts.next()?.trim().to_string();
                let val = parts.next()?.trim().to_string();
                Some((key, val))
            } else {
                None
            }
        })
        .collect();

    incoming
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            let lower = trimmed.to_ascii_lowercase();
            if lower.starts_with("rpcpassword=") || lower.starts_with("rpcuser=") {
                let key = trimmed.split('=').next().unwrap_or("");
                let val = trimmed.split('=').nth(1).unwrap_or("").trim();
                if val == REDACTED {
                    if let Some(secret) = secrets.get(key) {
                        return format!("{key}={secret}");
                    }
                }
            }
            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restore_conf_secrets_replaces_redacted_placeholders() {
        let existing = "server=1\nrpcuser=wallet_abc\nrpcpassword=secret-uuid\n";
        let incoming = "server=1\nrpcuser=***REDACTED***\nrpcpassword=***REDACTED***\n";
        let merged = restore_conf_secrets(incoming, existing);
        assert!(merged.contains("rpcuser=wallet_abc"));
        assert!(merged.contains("rpcpassword=secret-uuid"));
    }

    #[test]
    fn redact_conf_masks_rpc_secrets() {
        let raw = "server=1\nrpcuser=wallet_abc\nrpcpassword=secret-uuid\nrpcport=33987";
        let redacted = redact_conf_content(raw);
        assert!(redacted.contains("rpcuser=***REDACTED***"));
        assert!(redacted.contains("rpcpassword=***REDACTED***"));
        assert!(!redacted.contains("secret-uuid"));
    }
}
