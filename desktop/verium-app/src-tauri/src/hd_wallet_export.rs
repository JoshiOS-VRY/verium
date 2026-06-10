//! Export HD master keys from a full-node wallet dump (when no BIP39 backup exists).

use serde_json::json;

use crate::coin_profile::CoinId;
use crate::config;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::wallet::full_node_unlock::{ensure_full_wallet_unlock, RECOVERY_EXPORT_UNLOCK_SECONDS};

pub fn parse_xprv_from_dump(content: &str) -> AppResult<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("# extended private masterkey:") {
            let xprv = rest.trim();
            if xprv.is_empty() {
                continue;
            }
            return Ok(xprv.to_string());
        }
    }
    Err(AppError::other(
        "HD master key not found in wallet dump. The wallet may be non-HD or use imported keys only.",
    ))
}

pub async fn dump_hd_master_xprv(
    state: &AppState,
    coin: CoinId,
    unlock_passphrase: &str,
) -> AppResult<String> {
    if unlock_passphrase.is_empty() {
        return Err(AppError::other(
            "Wallet passphrase is required to export the HD master key.",
        ));
    }
    let cfg = state.config_fresh(coin).await?;
    let client = state.rpc_client(coin).await?;
    ensure_full_wallet_unlock(
        &client,
        coin,
        unlock_passphrase,
        RECOVERY_EXPORT_UNLOCK_SECONDS,
    )
    .await?;

    let backups_dir = config::wallet_backup_dir(coin, &cfg)?;
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
    client
        .call::<serde_json::Value>("dumpwallet", json!([filename]))
        .await?;

    let content = std::fs::read_to_string(&dump_path).map_err(|e| {
        let _ = std::fs::remove_file(&dump_path);
        AppError::other(format!("could not read wallet dump: {e}"))
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
