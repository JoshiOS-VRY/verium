//! Wallet RPC: info, addresses, send, sign/verify.

use crate::coin::CoinId;
use crate::context::AppContext;
use crate::error::{HostError, HostResult};
use crate::model::WalletInfo;
use crate::rpc::RpcClient;

pub async fn get_wallet_info(ctx: &AppContext, coin: CoinId) -> HostResult<WalletInfo> {
    if crate::light_session::is_light_mode(coin)? {
        return light_wallet_info(coin).await;
    }
    let endpoint = match ctx.endpoint(coin) {
        Some(ep) => ep,
        None => {
            return Ok(WalletInfo {
                missing: true,
                ..Default::default()
            })
        }
    };
    let client = RpcClient::new(ctx.http(), &endpoint);

    let info = match client.call("getwalletinfo", serde_json::json!([])).await {
        Ok(v) => v,
        Err(e) if e.is_warmup() => return Ok(WalletInfo::default()),
        Err(HostError::Http(_)) => {
            return Ok(WalletInfo {
                missing: true,
                ..Default::default()
            })
        }
        Err(e) => return Err(e),
    };

    let balance = info.get("balance").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let unconfirmed = info
        .get("unconfirmed_balance")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let immature = info
        .get("immature_balance")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let locked = matches!(
        info.get("unlocked_until").and_then(|v| v.as_i64()),
        Some(0)
    );
    let txcount = info.get("txcount").and_then(|v| v.as_i64()).unwrap_or(0);

    Ok(WalletInfo {
        balance,
        unconfirmed_balance: unconfirmed,
        immature_balance: immature,
        locked,
        missing: false,
        txcount,
    })
}

async fn light_wallet_info(coin: CoinId) -> HostResult<WalletInfo> {
    if !crate::light_session::light_wallet_exists(coin).await? {
        return Ok(WalletInfo {
            missing: true,
            ..Default::default()
        });
    }
    let bal = crate::light_session::light_wallet_balance(coin).await?;
    let confirmed = vericonomy_hd::sats_to_coins(bal.confirmed_sats);
    let unconfirmed = vericonomy_hd::sats_to_coins(bal.unconfirmed_sats);
    let immature = vericonomy_hd::sats_to_coins(bal.immature_sats);
    Ok(WalletInfo {
        balance: confirmed,
        unconfirmed_balance: unconfirmed,
        immature_balance: immature,
        locked: false,
        missing: false,
        txcount: 0,
    })
}

pub async fn get_new_address(ctx: &AppContext, coin: CoinId) -> HostResult<String> {
    let endpoint = ctx
        .endpoint(coin)
        .ok_or_else(|| HostError::other("no RPC endpoint"))?;
    let client = RpcClient::new(ctx.http(), &endpoint);
    let v = client.call("getnewaddress", serde_json::json!([])).await?;
    serde_json::from_value(v).map_err(Into::into)
}

pub async fn send_to_address(
    ctx: &AppContext,
    coin: CoinId,
    address: &str,
    amount: f64,
) -> HostResult<String> {
    let endpoint = ctx
        .endpoint(coin)
        .ok_or_else(|| HostError::other("no RPC endpoint"))?;
    let client = RpcClient::new(ctx.http(), &endpoint);
    let v = client
        .call(
            "sendtoaddress",
            serde_json::json!([address, amount, ""]),
        )
        .await?;
    serde_json::from_value(v).map_err(Into::into)
}

pub async fn sign_message(
    ctx: &AppContext,
    coin: CoinId,
    address: &str,
    message: &str,
) -> HostResult<String> {
    let endpoint = ctx
        .endpoint(coin)
        .ok_or_else(|| HostError::other("no RPC endpoint"))?;
    let client = RpcClient::new(ctx.http(), &endpoint);
    let v = client
        .call("signmessage", serde_json::json!([address, message]))
        .await?;
    serde_json::from_value(v).map_err(Into::into)
}

pub async fn verify_message(
    ctx: &AppContext,
    coin: CoinId,
    address: &str,
    signature: &str,
    message: &str,
) -> HostResult<bool> {
    let endpoint = ctx
        .endpoint(coin)
        .ok_or_else(|| HostError::other("no RPC endpoint"))?;
    let client = RpcClient::new(ctx.http(), &endpoint);
    let v = client
        .call(
            "verifymessage",
            serde_json::json!([address, signature, message]),
        )
        .await?;
    serde_json::from_value(v).map_err(Into::into)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WalletCreateResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WalletRestoreResult {
    pub success: bool,
    pub message: String,
    pub destination: String,
    pub rescan_started: bool,
}

pub async fn wallet_unlock(
    ctx: &AppContext,
    coin: CoinId,
    passphrase: &str,
    timeout_seconds: i64,
) -> HostResult<()> {
    if crate::light_session::is_light_mode(coin)? {
        return crate::light_session::light_wallet_unlock(coin, passphrase).await;
    }
    let endpoint = ctx
        .endpoint(coin)
        .ok_or_else(|| HostError::other("no RPC endpoint"))?;
    let client = RpcClient::new(ctx.http(), &endpoint);
    let _: serde_json::Value = client
        .call(
            "walletpassphrase",
            serde_json::json!([passphrase, timeout_seconds.max(1)]),
        )
        .await?;
    Ok(())
}

pub async fn wallet_create_encrypted(
    ctx: &AppContext,
    coin: CoinId,
    passphrase: &str,
) -> HostResult<WalletCreateResult> {
    if passphrase.is_empty() {
        return Err(HostError::other("passphrase must not be empty"));
    }
    let endpoint = ctx
        .endpoint(coin)
        .ok_or_else(|| HostError::other("no RPC endpoint"))?;
    let client = RpcClient::new(ctx.http(), &endpoint);
    let v = client
        .call("encryptwallet", serde_json::json!([passphrase]))
        .await;
    match v {
        Ok(val) => {
            let msg = val.as_str().unwrap_or("Wallet encrypted.").to_string();
            Ok(WalletCreateResult {
                success: true,
                message: msg,
            })
        }
        Err(HostError::Http(_)) => Ok(WalletCreateResult {
            success: true,
            message: "Wallet encrypted; restart the daemon if it stopped.".into(),
        }),
        Err(e) => Err(e),
    }
}

pub async fn wallet_restore(
    ctx: &AppContext,
    coin: CoinId,
    source_path: &str,
) -> HostResult<WalletRestoreResult> {
    if source_path.trim().is_empty() {
        return Err(HostError::other("source_path must not be empty"));
    }
    let source = std::path::PathBuf::from(source_path);
    if !source.is_file() {
        return Err(HostError::other(format!("Backup file not found: {source_path}")));
    }
    let meta = std::fs::metadata(&source)?;
    if meta.len() < 512 {
        return Err(HostError::other(
            "Selected file is too small to be a valid wallet.dat backup.",
        ));
    }
    let cfg = crate::daemon_config::load_daemon_config(coin)?;
    if crate::daemon_config::is_live_wallet_destination(coin, &cfg, &source) {
        return Err(HostError::other(
            "That file is the live wallet.dat already in use.",
        ));
    }
    let dest = crate::daemon_config::wallet_dat_path(coin, &cfg);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if dest.exists() {
        let backup_dir = crate::daemon_config::wallet_backup_dir(coin, &cfg)?;
        let stamp = chrono_lite_timestamp();
        let pre = backup_dir.join(format!("pre-restore-{stamp}.dat"));
        std::fs::copy(&dest, &pre)?;
    }
    crate::daemon_config::clear_wallet_bdb_environment(&dest)?;
    std::fs::copy(&source, &dest)?;

    let mut rescan_started = false;
    if let Some(ep) = ctx.endpoint(coin) {
        let client = RpcClient::new(ctx.http(), &ep);
        if client
            .call("rescanblockchain", serde_json::json!([0]))
            .await
            .is_ok()
        {
            rescan_started = true;
        }
    }

    let mut message = String::from(
        "Wallet restored. Unlock with the passphrase from when that backup was made.",
    );
    if rescan_started {
        message.push_str(" Background rescan started.");
    }

    Ok(WalletRestoreResult {
        success: true,
        message,
        destination: dest.display().to_string(),
        rescan_started,
    })
}

fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}
