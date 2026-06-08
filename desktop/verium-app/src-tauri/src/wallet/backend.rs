//! Chain backend resolution for light vs full-node mode.

use std::sync::Arc;

use crate::chain::electrum::ElectrumLightClient;
use crate::chain::full_node::FullNodeRpcClient;
use crate::chain::ChainBackend;
use crate::coin_profile::CoinId;
use crate::error::AppResult;
use crate::features::effective_network_mode;
use crate::prefs;
use crate::state::AppState;

pub async fn resolve_backend(state: &AppState, coin: CoinId) -> AppResult<Arc<dyn ChainBackend>> {
    let prefs = prefs::load().await?;
    let network = effective_network_mode(prefs.network_mode);

    if prefs::wallet_mode_for(&prefs, coin).is_light() {
        let servers = prefs
            .electrum_servers_by_coin
            .as_ref()
            .and_then(|m| m.get(coin.as_str()).cloned())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| coin.default_electrum_servers(network));
        let client = ElectrumLightClient::new(coin, &servers)?;
        Ok(Arc::new(client))
    } else {
        let rpc = state.rpc_client(coin).await?;
        Ok(Arc::new(FullNodeRpcClient::new(coin, rpc)))
    }
}
