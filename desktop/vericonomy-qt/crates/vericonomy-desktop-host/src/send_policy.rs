//! Send authorization — 2FA, spending controls, allowlist (Tauri `security_policy` subset).

use crate::coin::CoinId;
use crate::commands::addressbook;
use crate::context::AppContext;
use crate::error::{HostError, HostResult};
use crate::rpc::RpcClient;
use crate::spending_controls::{self, SpendCheckResult};

pub fn require_gated_action(action: &str, totp_code: Option<&str>) -> HostResult<()> {
    if !crate::two_factor::is_action_gated(action, None, "")? {
        return Ok(());
    }
    let code = totp_code
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| HostError::other(format!("2FA code required for {action}")))?;
    if !crate::two_factor::verify(code)? {
        return Err(HostError::other("Invalid 2FA code"));
    }
    Ok(())
}

pub fn authorize_send(
    coin: CoinId,
    amount: f64,
    totp_code: Option<&str>,
) -> HostResult<()> {
    if crate::two_factor::is_action_gated("send", Some(amount), coin.as_str())? {
        return require_gated_action("send", totp_code);
    }
    Ok(())
}

pub async fn ensure_wallet_unlocked_for_send(
    ctx: &AppContext,
    coin: CoinId,
    passphrase: &str,
) -> HostResult<()> {
    if crate::light_session::is_light_mode(coin)? {
        if !crate::light_session::light_wallet_is_unlocked(coin).await? {
            crate::light_session::light_wallet_unlock(coin, passphrase).await?;
        }
        return Ok(());
    }
    let pass = passphrase.trim();
    if pass.is_empty() {
        return Err(HostError::other("Wallet passphrase required to send"));
    }
    let endpoint = ctx
        .endpoint(coin)
        .ok_or_else(|| HostError::other("no RPC endpoint"))?;
    let client = RpcClient::new(ctx.http(), &endpoint);
    let info = client.call("getwalletinfo", serde_json::json!([])).await?;
    if info.get("unlocked_until").is_none() {
        return Ok(());
    }
    let _ = client.call("walletlock", serde_json::json!([])).await;
    client
        .call(
            "walletpassphrase",
            serde_json::json!([pass, 120_i64]),
        )
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.to_ascii_lowercase().contains("passphrase") {
                HostError::other("Incorrect wallet passphrase")
            } else {
                e
            }
        })?;
    Ok(())
}

pub fn check_send_allowed(
    coin: CoinId,
    amount: f64,
    address: &str,
    extra_confirmed: bool,
) -> HostResult<SpendCheckResult> {
    let config = spending_controls::load().unwrap_or_default();
    let wallet_already_sent =
        spending_controls::is_known_send_destination(coin.as_str(), address, &config);

    let check = spending_controls::check_spend_allowed(
        amount,
        coin.as_str(),
        address,
        wallet_already_sent,
    )?;
    if !check.allowed {
        return Ok(check);
    }
    if check.requires_extra_confirmation && !extra_confirmed {
        return Err(HostError::other(
            "First send to this address requires explicit confirmation",
        ));
    }

    if config.allowlist_only {
        let entries = addressbook::list(coin)?;
        let allowlist: Vec<String> = entries
            .iter()
            .filter(|e| e.category.eq_ignore_ascii_case("send"))
            .map(|e| e.address.clone())
            .collect();
        if !spending_controls::check_allowlist(address, &allowlist) {
            return Ok(SpendCheckResult {
                allowed: false,
                reason: Some(
                    "Allowlist mode: add this address to your address book (Send) first".into(),
                ),
                requires_extra_confirmation: false,
                look_alike_warning: None,
            });
        }
    }

    Ok(check)
}

pub fn record_send(coin: CoinId, amount: f64, address: &str) -> HostResult<()> {
    spending_controls::record_spend(amount, coin.as_str(), address)
}
