//! Light-wallet host commands (xprv / phrase import, unlock, sync).

use crate::coin::CoinId;
use crate::context::AppContext;
use crate::error::HostResult;
use crate::light_session;

pub async fn exists(_ctx: &AppContext, coin: CoinId) -> HostResult<bool> {
    light_session::light_wallet_exists(coin).await
}

pub async fn import_wallet(
    _ctx: &AppContext,
    coin: CoinId,
    seed_secret: &str,
    passphrase: &str,
    label: Option<&str>,
) -> HostResult<()> {
    light_session::light_wallet_import(coin, seed_secret, passphrase, label).await
}

pub async fn unlock(_ctx: &AppContext, coin: CoinId, passphrase: &str) -> HostResult<()> {
    light_session::light_wallet_unlock(coin, passphrase).await
}

pub async fn sync(_ctx: &AppContext, coin: CoinId) -> HostResult<()> {
    light_session::light_wallet_sync(coin).await
}
