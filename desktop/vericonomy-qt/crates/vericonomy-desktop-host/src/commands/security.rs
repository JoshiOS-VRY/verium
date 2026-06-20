//! Security commands (P2) — 2FA, backups, spending controls.

use crate::coin::CoinId;
use crate::context::AppContext;
use crate::error::{HostError, HostResult};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackupHealth {
    pub backup_count: u32,
    pub last_backup_at: Option<i64>,
}

pub async fn backup_health(_ctx: &AppContext, _coin: CoinId) -> HostResult<BackupHealth> {
    // TODO: port backup_scheduler + mnemonic_backup from verium-app security_commands
    Ok(BackupHealth {
        backup_count: 0,
        last_backup_at: None,
    })
}

pub async fn two_factor_status() -> HostResult<bool> {
    Err(HostError::other("2FA not yet ported — use Tauri Security until P2 completes"))
}
