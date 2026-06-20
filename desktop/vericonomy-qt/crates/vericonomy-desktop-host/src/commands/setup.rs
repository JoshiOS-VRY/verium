//! Setup status + completion.

use crate::coin::CoinId;
use crate::context::AppContext;
use crate::daemon_config;
use crate::error::HostResult;
use crate::model::SetupStatus;
use crate::prefs::{load_prefs, save_prefs, UserPreferences};

pub async fn get_setup_status(ctx: &AppContext, coin: CoinId) -> HostResult<SetupStatus> {
    let prefs = load_prefs().unwrap_or_default();
    let is_light = crate::light_session::is_light_mode(coin).unwrap_or(false);
    let cfg = daemon_config::load_daemon_config(coin)?;
    let has_full_node_wallet = daemon_config::wallet_dat_exists(coin, &cfg);
    let has_light_wallet = crate::light_session::light_wallet_exists(coin)
        .await
        .unwrap_or(false);

    let wallet = if is_light {
        has_light_wallet
    } else if has_full_node_wallet {
        true
    } else {
        crate::commands::wallet::get_wallet_info(ctx, coin)
            .await
            .map(|w| !w.missing)
            .unwrap_or(false)
    };
    let node = crate::commands::node::get_node_status(ctx, coin).await?;
    Ok(SetupStatus {
        setup_completed: prefs.setup_completed,
        wallet_ready: wallet,
        node_reachable: node.connected,
        has_full_node_wallet,
        has_light_wallet,
        is_light_mode: is_light,
    })
}

pub fn complete_setup() -> HostResult<()> {
    let mut prefs = load_prefs().unwrap_or_default();
    prefs.setup_completed = true;
    save_prefs(&prefs)
}

pub fn get_prefs() -> HostResult<UserPreferences> {
    load_prefs()
}

pub fn save_user_prefs(prefs: &UserPreferences) -> HostResult<()> {
    save_prefs(prefs)
}
