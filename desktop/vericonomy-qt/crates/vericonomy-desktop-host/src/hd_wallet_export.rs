//! Export HD master keys from a full-node wallet dump (when no BIP39 backup exists).

use serde_json::json;

use crate::coin::CoinId;
use crate::daemon_config;
use crate::error::{HostError, HostResult};
use crate::rpc::RpcClient;
use crate::context::AppContext;

const RECOVERY_EXPORT_UNLOCK_SECONDS: i64 = 120;

pub fn parse_xprv_from_dump(content: &str) -> HostResult<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("# extended private masterkey:") {
            let xprv = rest.trim();
            if !xprv.is_empty() {
                return Ok(xprv.to_string());
            }
        }
    }
    Err(HostError::other(
        "HD master key not found in wallet dump. The wallet may be non-HD or use imported keys only.",
    ))
}

async fn ensure_wallet_unlocked(
    client: &RpcClient<'_>,
    passphrase: &str,
) -> HostResult<()> {
    let info = client.call("getwalletinfo", json!([])).await?;
    let needs_unlock = matches!(
        info.get("unlocked_until").and_then(|v| v.as_i64()),
        Some(0) | None
    );
    if !needs_unlock {
        return Ok(());
    }
    let _: serde_json::Value = client
        .call(
            "walletpassphrase",
            json!([passphrase, RECOVERY_EXPORT_UNLOCK_SECONDS]),
        )
        .await?;
    Ok(())
}

pub async fn dump_hd_master_xprv(
    ctx: &AppContext,
    coin: CoinId,
    unlock_passphrase: &str,
) -> HostResult<String> {
    if unlock_passphrase.is_empty() {
        return Err(HostError::other(
            "Wallet passphrase is required to export the HD master key.",
        ));
    }
    let cfg = daemon_config::load_daemon_config(coin)?;
    let endpoint = ctx
        .endpoint(coin)
        .ok_or_else(|| HostError::other("no RPC endpoint"))?;
    let client = RpcClient::new(ctx.http(), &endpoint);
    ensure_wallet_unlocked(&client, unlock_passphrase).await?;

    let backups_dir = daemon_config::wallet_backup_dir(coin, &cfg)?;
    std::fs::create_dir_all(&backups_dir)?;
    let dump_path = backups_dir.join(format!(
        "vericonomy-hd-export-{}.tmp",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    if dump_path.exists() {
        std::fs::remove_file(&dump_path)?;
    }

    let filename = dump_path.to_string_lossy().to_string();
    client.call("dumpwallet", json!([filename])).await?;

    let content = std::fs::read_to_string(&dump_path).map_err(|e| {
        let _ = std::fs::remove_file(&dump_path);
        HostError::other(format!("could not read wallet dump: {e}"))
    })?;
    let _ = std::fs::remove_file(&dump_path);

    parse_xprv_from_dump(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_extended_master_from_dump() {
        let sample = r#"# Wallet dump created by Verium
# extended private masterkey: xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPsjiXazd8RVd9XF7

abc123 2020-01-01T00:00:00Z hdseed=1 # addr=...
"#;
        let xprv = parse_xprv_from_dump(sample).unwrap();
        assert!(xprv.starts_with("xprv"));
    }
}
