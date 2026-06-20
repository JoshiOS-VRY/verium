//! `AppContext` — the owned application state that replaces Tauri's managed
//! `State<'_, AppState>`.
//!
//! The Qt bridge constructs one `Arc<AppContext>` at startup and hands clones to
//! every controller. It owns the HTTP client, the per-coin RPC endpoints, and
//! the `HostBridge` used for side effects. In Phase 1b this grows to own the
//! `DaemonManager`/`ProcessRegistry` ported from
//! `desktop/verium-app/src-tauri/src/state.rs`.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::bridge::{NullHostBridge, SharedHostBridge};
use crate::coin::CoinId;
use crate::model::EarnState;
use crate::rpc::RpcEndpoint;

pub struct AppContext {
    bridge: SharedHostBridge,
    http: reqwest::Client,
    endpoints: RwLock<HashMap<CoinId, RpcEndpoint>>,
    earn: RwLock<HashMap<CoinId, EarnState>>,
}

impl AppContext {
    pub fn new(bridge: SharedHostBridge) -> Arc<Self> {
        Arc::new(Self {
            bridge,
            http: reqwest::Client::new(),
            endpoints: RwLock::new(HashMap::new()),
            earn: RwLock::new(HashMap::new()),
        })
    }

    /// Convenience constructor for headless/test use.
    pub fn headless() -> Arc<Self> {
        Self::new(Arc::new(NullHostBridge))
    }

    pub fn bridge(&self) -> &SharedHostBridge {
        &self.bridge
    }

    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    /// Register/replace the RPC endpoint for a coin (called after the daemon
    /// config is resolved in Phase 1b; settable directly for now).
    pub fn set_endpoint(&self, coin: CoinId, endpoint: RpcEndpoint) {
        self.endpoints
            .write()
            .expect("endpoints lock poisoned")
            .insert(coin, endpoint);
    }

    pub fn endpoint(&self, coin: CoinId) -> Option<RpcEndpoint> {
        self.endpoints
            .read()
            .expect("endpoints lock poisoned")
            .get(&coin)
            .cloned()
    }

    /// Populate endpoints for both coins from the on-disk `vericonomy.conf`.
    /// Coins without resolvable credentials are left unset (reported "Offline").
    pub fn load_endpoints_from_conf(&self) {
        for coin in [CoinId::Verium, CoinId::Vericoin] {
            if let Some(ep) = crate::config::resolve_endpoint(coin) {
                self.set_endpoint(coin, ep);
            }
        }
    }

    pub fn earn(&self, coin: CoinId) -> EarnState {
        self.earn
            .read()
            .expect("earn lock")
            .get(&coin)
            .cloned()
            .unwrap_or_default()
    }

    pub fn set_earn(&self, coin: CoinId, state: EarnState) {
        self.earn
            .write()
            .expect("earn lock")
            .insert(coin, state);
    }
}
