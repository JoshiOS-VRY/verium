//! Per-coin wallet mode (full node vs light).

use serde::{Deserialize, Serialize};
use vericonomy_wallet_engine::WalletMode;

use crate::coin::CoinId;
use crate::context::AppContext;
use crate::error::HostResult;
use crate::prefs::{self, load_prefs, save_prefs, set_wallet_mode_for};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletModeStatus {
    pub mode: String,
    pub light_wallet_enabled: bool,
    pub light_wallet_exists: bool,
    pub electrum_servers: Vec<String>,
    pub mobile_only: bool,
}

fn status_for(prefs: &prefs::UserPreferences, coin: CoinId) -> WalletModeStatus {
    WalletModeStatus {
        mode: prefs::wallet_mode_for(prefs, coin).as_str().to_string(),
        light_wallet_enabled: true,
        light_wallet_exists: false,
        electrum_servers: prefs::electrum_servers_for(prefs, coin),
        mobile_only: false,
    }
}

pub async fn get_for_coin(_ctx: &AppContext, coin: CoinId) -> HostResult<WalletModeStatus> {
    let prefs = load_prefs()?;
    let mut status = status_for(&prefs, coin);
    status.light_wallet_exists = crate::light_session::light_wallet_exists(coin)
        .await
        .unwrap_or(false);
    Ok(status)
}

pub fn set_for_coin(coin: CoinId, mode: &str) -> HostResult<()> {
    let parsed = WalletMode::from_str_lossy(mode);
    let mut prefs = load_prefs()?;
    if parsed.is_light() {
        set_wallet_mode_for(&mut prefs, coin, WalletMode::Light);
    } else {
        set_wallet_mode_for(&mut prefs, coin, WalletMode::FullNode);
    }
    save_prefs(&prefs)
}
