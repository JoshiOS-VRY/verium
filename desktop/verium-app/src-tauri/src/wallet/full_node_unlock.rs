//! Full-node wallet unlock helpers (minting-only vs full signing access).

use serde_json::{json, Value};

use crate::coin_profile::CoinId;
use crate::error::AppResult;
use crate::rpc::RpcClient;

/// Temporary full unlock for recovery export / restore (10 minutes).
pub const RECOVERY_EXPORT_UNLOCK_SECONDS: i64 = 600;

/// True when the wallet is fully locked (`unlocked_until` is 0 or expired).
pub fn wallet_info_is_locked(info: &Value) -> bool {
    let Some(until) = info.get("unlocked_until").and_then(|v| v.as_i64()) else {
        return false;
    };
    if until == 0 {
        return true;
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    until <= now
}

/// Vericoin reports `unlocked_minting_only` after auto-stake / mint-only unlock.
pub fn wallet_info_minting_only(info: &Value) -> bool {
    info.get("unlocked_minting_only")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Export, send, and dumpwallet need full unlock — not minting-only.
pub fn wallet_needs_full_unlock(info: &Value) -> bool {
    wallet_info_is_locked(info) || wallet_info_minting_only(info)
}

pub fn walletpassphrase_params(coin: CoinId, passphrase: &str, seconds: i64) -> Value {
    match coin {
        // Third param `false` = not minting-only (Vericoin walletpassphrase).
        CoinId::Vericoin => json!([passphrase, seconds, false]),
        CoinId::Verium => json!([passphrase, seconds]),
    }
}

/// Ensure the wallet can sign and dump keys. Upgrades minting-only unlock in place.
pub async fn ensure_full_wallet_unlock(
    client: &RpcClient,
    coin: CoinId,
    passphrase: &str,
    seconds: i64,
) -> AppResult<()> {
    let info: Value = client.call("getwalletinfo", json!([])).await?;
    if !wallet_needs_full_unlock(&info) {
        return Ok(());
    }
    client
        .call_no_result(
            "walletpassphrase",
            walletpassphrase_params(coin, passphrase, seconds),
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minting_only_wallet_needs_full_unlock() {
        let info = json!({
            "unlocked_until": 9999999999_i64,
            "unlocked_minting_only": true,
        });
        assert!(!wallet_info_is_locked(&info));
        assert!(wallet_info_minting_only(&info));
        assert!(wallet_needs_full_unlock(&info));
    }

    #[test]
    fn fully_unlocked_wallet_does_not_need_reunlock() {
        let info = json!({
            "unlocked_until": 9999999999_i64,
            "unlocked_minting_only": false,
        });
        assert!(!wallet_needs_full_unlock(&info));
    }
}
