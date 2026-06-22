//! Bootstrap import commands.

use std::path::PathBuf;

use crate::bootstrap::{self, BootstrapResult};
use crate::coin::CoinId;
use crate::context::AppContext;
use crate::error::HostResult;
use crate::prefs;

pub async fn import_bootstrap(
    ctx: &AppContext,
    coin: CoinId,
    local_path: Option<PathBuf>,
) -> HostResult<BootstrapResult> {
    let result = bootstrap::import_bootstrap(ctx, coin, local_path).await?;
    if result.success {
        mark_bootstrap_imported(coin)?;
    }
    Ok(result)
}

pub fn cancel_bootstrap(coin: CoinId) {
    bootstrap::cancel_bootstrap(coin);
}

fn mark_bootstrap_imported(coin: CoinId) -> HostResult<()> {
    let mut prefs = prefs::load_prefs()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let key = "bootstrap_imported_at_by_coin";
    let mut map = prefs
        .extra
        .get(key)
        .and_then(|v| serde_json::from_value::<std::collections::HashMap<String, i64>>(v.clone()).ok())
        .unwrap_or_default();
    map.insert(coin.as_str().to_string(), now);
    prefs
        .extra
        .insert(key.to_string(), serde_json::to_value(map).unwrap_or_default());
    prefs::save_prefs(&prefs)
}
