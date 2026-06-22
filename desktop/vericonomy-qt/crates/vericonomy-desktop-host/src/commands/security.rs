//! Security commands (P2) — recovery phrase, payment URIs, backups.

use crate::coin::CoinId;
use crate::context::AppContext;
use crate::error::{HostError, HostResult};
use crate::prefs::CoinSdkExt;
use crate::rpc::RpcClient;
use vericonomy_chain_params::CoinProfile;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackupHealth {
    pub backup_count: u32,
    pub last_backup_at: Option<i64>,
}

pub async fn backup_health(_ctx: &AppContext, _coin: CoinId) -> HostResult<BackupHealth> {
    Ok(BackupHealth {
        backup_count: 0,
        last_backup_at: None,
    })
}

pub async fn two_factor_status() -> HostResult<crate::two_factor::TwoFactorConfig> {
    crate::two_factor::status()
}

pub fn two_factor_verify(code: &str) -> HostResult<bool> {
    crate::two_factor::verify(code)
}

pub fn two_factor_start_enrollment() -> HostResult<crate::two_factor::TwoFactorEnrollment> {
    crate::two_factor::start_enrollment()
}

pub fn two_factor_confirm_enrollment(
    code: &str,
    enrollment_secret: &str,
) -> HostResult<crate::two_factor::TwoFactorConfig> {
    crate::two_factor::confirm_enrollment(code, Some(enrollment_secret))
}

pub fn two_factor_disable(code: &str) -> HostResult<()> {
    crate::two_factor::disable(code)
}

pub fn two_factor_is_gated(action: &str, amount: Option<f64>, coin: CoinId) -> HostResult<bool> {
    crate::two_factor::is_action_gated(action, amount, coin.as_str())
}

pub fn auto_lock_get_config() -> HostResult<crate::auto_lock::AutoLockConfig> {
    crate::auto_lock::load_config()
}

pub fn auto_lock_set_config(config: &crate::auto_lock::AutoLockConfig) -> HostResult<()> {
    crate::auto_lock::save_config(config)
}

pub fn auto_lock_record_activity() {
    crate::auto_lock::record_activity();
}

pub fn auto_lock_should_lock() -> HostResult<bool> {
    let config = crate::auto_lock::load_config()?;
    Ok(crate::auto_lock::should_auto_lock(&config))
}

pub fn spending_controls_get() -> HostResult<crate::spending_controls::SpendingControlsConfig> {
    crate::spending_controls::load()
}

pub fn spending_controls_save(
    config: &crate::spending_controls::SpendingControlsConfig,
) -> HostResult<()> {
    crate::spending_controls::save_ui_settings(config)
}

pub fn spending_controls_check_send(
    coin: CoinId,
    address: &str,
    amount: f64,
) -> HostResult<crate::spending_controls::SpendCheckResult> {
    let config = crate::spending_controls::load().unwrap_or_default();
    let wallet_already_sent =
        crate::spending_controls::is_known_send_destination(coin.as_str(), address, &config);
    crate::spending_controls::check_send_with_allowlist(coin, address, amount, wallet_already_sent)
}

pub fn spending_controls_record_send(coin: CoinId, address: &str, amount: f64) -> HostResult<()> {
    crate::send_policy::record_send(coin, amount, address)
}

pub fn receive_requests_list(coin: CoinId) -> HostResult<Vec<crate::receive_requests::ReceiveRequest>> {
    crate::receive_requests::list(coin)
}

pub fn receive_requests_append(
    coin: CoinId,
    entry: crate::receive_requests::ReceiveRequest,
) -> HostResult<crate::receive_requests::ReceiveRequest> {
    crate::receive_requests::append(coin, entry)
}

pub fn receive_requests_delete(coin: CoinId, id: &str) -> HostResult<()> {
    crate::receive_requests::delete_entry(coin, id)
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RecoveryExportResult {
    Mnemonic {
        mnemonic: String,
        word_count: u32,
    },
    HdMasterXprv {
        xprv: String,
        message: String,
    },
}

pub async fn recovery_export_seed(
    ctx: &AppContext,
    coin: CoinId,
    wallet_passphrase: &str,
) -> HostResult<RecoveryExportResult> {
    if wallet_passphrase.is_empty() {
        return Err(HostError::other(
            "Wallet passphrase is required to export your recovery material.",
        ));
    }
    if crate::light_session::is_light_mode(coin)? {
        if !crate::light_session::light_wallet_exists(coin).await? {
            return Err(HostError::other("No light wallet found for this chain."));
        }
        if !crate::light_session::light_wallet_is_unlocked(coin).await? {
            crate::light_session::light_wallet_unlock(coin, wallet_passphrase).await?;
        }
        let secret = crate::light_session::light_wallet_export_secret(coin, wallet_passphrase).await?;
        let sdk = coin.to_sdk();
        if vericonomy_hd::is_hd_master_secret(sdk, &secret) {
            return Ok(RecoveryExportResult::HdMasterXprv {
                xprv: secret,
                message: "Light wallet was imported from an HD master key (xprv), not a BIP39 phrase."
                    .into(),
            });
        }
        let word_count = secret.split_whitespace().count() as u32;
        return Ok(RecoveryExportResult::Mnemonic {
            mnemonic: secret,
            word_count,
        });
    }

    if recovery_wallet_is_hd(ctx, coin).await? {
        let xprv = crate::hd_wallet_export::dump_hd_master_xprv(
            ctx,
            coin,
            wallet_passphrase,
        )
        .await?;
        return Ok(RecoveryExportResult::HdMasterXprv {
            xprv,
            message: "No BIP39 phrase was stored when this wallet was upgraded. Use this xprv to import into light mode, or set up a new phrase under Security (back up wallet.dat first).".into(),
        });
    }

    Err(HostError::other(
        "This wallet is not HD. Upgrade to HD under Security (back up wallet.dat first), or continue using full-node mode with wallet.dat backups.",
    ))
}

fn coin_profile(coin: CoinId) -> CoinProfile {
    let sdk_coin = match coin {
        CoinId::Verium => vericonomy_chain_params::CoinId::Verium,
        CoinId::Vericoin => vericonomy_chain_params::CoinId::Vericoin,
    };
    CoinProfile::for_coin(sdk_coin)
}

fn wallet_needs_unlock(info: &serde_json::Value) -> bool {
    matches!(
        info.get("unlocked_until").and_then(|v| v.as_i64()),
        Some(0) | None
    )
}

/// Apply a BIP39 recovery phrase to a full-node wallet via `sethdseed`.
pub async fn recovery_apply_hd_seed(
    ctx: &AppContext,
    coin: CoinId,
    phrase: &str,
    bip39_passphrase: Option<&str>,
    unlock_passphrase: Option<&str>,
) -> HostResult<String> {
    if crate::light_session::is_light_mode(coin)? {
        return Err(HostError::other(
            "Recovery phrase apply is only supported in full-node mode.",
        ));
    }
    let profile = coin_profile(coin);
    let wif = vericonomy_wallet_core::master_xpriv_to_wif(
        profile.wif_secret_prefix,
        phrase,
        bip39_passphrase,
    )
    .map_err(|e| HostError::other(e.to_string()))?;

    let endpoint = ctx
        .endpoint(coin)
        .ok_or_else(|| HostError::other("no RPC endpoint"))?;
    let client = RpcClient::new(ctx.http(), &endpoint);

    let info = client.call("getwalletinfo", serde_json::json!([])).await?;
    if wallet_needs_unlock(&info) {
        let pass = unlock_passphrase
            .filter(|p| !p.is_empty())
            .ok_or_else(|| {
                HostError::other(
                    "Wallet is locked. Enter your wallet passphrase to apply the recovery phrase.",
                )
            })?;
        let _: serde_json::Value = client
            .call(
                "walletpassphrase",
                serde_json::json!([pass, 3600_i64]),
            )
            .await?;
    }

    client
        .call("sethdseed", serde_json::json!([true, wif]))
        .await?;

    let after = client.call("getwalletinfo", serde_json::json!([])).await?;
    if after.get("hdseedid").is_none() {
        return Err(HostError::other(
            "Recovery seed was not applied (wallet is still non-HD). Back up wallet.dat, then try again.",
        ));
    }

    Ok("HD seed applied. Back up wallet.dat immediately.".into())
}

pub async fn recovery_wallet_is_hd(ctx: &AppContext, coin: CoinId) -> HostResult<bool> {
    if crate::light_session::is_light_mode(coin)? {
        return Ok(true);
    }
    let endpoint = ctx
        .endpoint(coin)
        .ok_or_else(|| HostError::other("no RPC endpoint"))?;
    let client = RpcClient::new(ctx.http(), &endpoint);
    let info = client.call("getwalletinfo", serde_json::json!([])).await?;
    Ok(info.get("hdseedid").is_some())
}

pub fn build_payment_uri(
    coin: CoinId,
    address: &str,
    amount: Option<f64>,
    label: Option<&str>,
    message: Option<&str>,
) -> HostResult<String> {
    let scheme = match coin {
        CoinId::Vericoin => "vericoin",
        CoinId::Verium => "verium",
    };
    let mut uri = format!("{scheme}:{address}");
    let mut params = Vec::new();
    if let Some(a) = amount {
        params.push(format!("amount={a}"));
    }
    if let Some(l) = label.filter(|s| !s.is_empty()) {
        params.push(format!("label={}", urlencoding::encode(l)));
    }
    if let Some(m) = message.filter(|s| !s.is_empty()) {
        params.push(format!("message={}", urlencoding::encode(m)));
    }
    if !params.is_empty() {
        uri.push('?');
        uri.push_str(&params.join("&"));
    }
    Ok(uri)
}
