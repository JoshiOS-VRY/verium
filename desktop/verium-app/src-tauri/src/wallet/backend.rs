//! Chain backend resolution for light vs full-node mode.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;

use crate::chain::electrum::ElectrumLightClient;
use crate::chain::full_node::FullNodeRpcClient;
use crate::chain::ChainBackend;
use crate::coin_profile::CoinId;
use crate::error::{AppError, AppResult};
use crate::prefs;
use crate::state::AppState;

/// Process-wide pool of light-mode Electrum clients, keyed by coin. Each client
/// owns a reused TCP/TLS connection (lazily established, auto-reconnecting), so
/// pooling avoids a fresh handshake on every `resolve_backend` call. The stored
/// server list is part of the key: if the user changes Electrum servers in
/// prefs, the next resolve rebuilds the client. Concurrent callers are safe —
/// `ElectrumConnection` serializes requests on an internal io mutex.
static ELECTRUM_POOL: Lazy<Mutex<HashMap<CoinId, (Vec<String>, Arc<ElectrumLightClient>)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn pooled_electrum_client(
    coin: CoinId,
    servers: &[String],
) -> AppResult<Arc<ElectrumLightClient>> {
    let mut pool = ELECTRUM_POOL
        .lock()
        .map_err(|_| AppError::other("electrum client pool poisoned"))?;
    if let Some((cached_servers, client)) = pool.get(&coin) {
        if cached_servers.as_slice() == servers {
            return Ok(client.clone());
        }
    }
    let client = Arc::new(ElectrumLightClient::new(coin, servers)?);
    pool.insert(coin, (servers.to_vec(), client.clone()));
    Ok(client)
}

/// Drop any pooled Electrum client for a coin (e.g. on wallet lock or shutdown),
/// closing its connection.
pub fn drop_pooled_electrum_client(coin: CoinId) {
    if let Ok(mut pool) = ELECTRUM_POOL.lock() {
        pool.remove(&coin);
    }
    crate::chain::electrum::throttle::clear_cooldown(coin);
}

/// Drop all pooled Electrum clients (app shutdown / teardown), closing their
/// connections so no sockets are leaked when the wallet exits.
pub fn drop_all_pooled_electrum_clients() {
    if let Ok(mut pool) = ELECTRUM_POOL.lock() {
        pool.clear();
    }
}

pub async fn resolve_backend(state: &AppState, coin: CoinId) -> AppResult<Arc<dyn ChainBackend>> {
    let prefs = prefs::load().await?;

    if prefs::wallet_mode_for(&prefs, coin).is_light() {
        let servers = prefs::electrum_servers_for(&prefs, coin);
        let client = pooled_electrum_client(coin, &servers)?;
        Ok(client as Arc<dyn ChainBackend>)
    } else {
        let rpc = state.rpc_client(coin).await?;
        Ok(Arc::new(FullNodeRpcClient::new(coin, rpc)))
    }
}
