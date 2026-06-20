//! `vericonomy-desktop-host` — the thin, desktop-only host layer for the Qt shell.
//!
//! Responsibilities (per the Qt/QML port plan, layer 2):
//! - Own `AppContext` (the Tauri-free replacement for managed `AppState`).
//! - Define `HostBridge`, the abstraction that replaces Tauri's `AppHandle` for
//!   events / OS integration.
//! - Host desktop-only concerns: daemon subprocess lifecycle, full-node RPC
//!   management, OS keychain, updater, deep links (added incrementally in 1b).
//!
//! Platform-agnostic wallet/security logic lives in `vericonomy-sdk` (the single
//! source of truth shared with the native iOS app); this crate orchestrates it
//! for the desktop.

pub mod bridge;
pub mod coin;
pub mod commands;
pub mod config;
pub mod context;
pub mod daemon_config;
pub mod error;
pub mod light_session;
pub mod model;
pub mod os;
pub mod prefs;
pub mod rpc;

pub use bridge::{DesktopHostBridge, HostBridge, NullHostBridge, SharedHostBridge};
pub use coin::CoinId;
pub use context::AppContext;
pub use error::{HostError, HostResult};
pub use model::{
    BalancePoint, DashboardSnapshot, EarnState, ExplorerBlock, ExplorerStats, LogLine, MiningInfo,
    NodeStatus, PeerRow, SetupStatus, StakingInfo, TransactionRow, WalletInfo,
};
pub use prefs::UserPreferences;
pub use vericonomy_wallet_engine::WalletMode;
pub use rpc::{RpcClient, RpcEndpoint};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn offline_daemon_reports_disconnected() {
        let ctx = AppContext::headless();
        // Point at a port with no listener.
        ctx.set_endpoint(
            CoinId::Verium,
            RpcEndpoint::loopback(59999, "u", "p"),
        );
        let status = commands::node::get_node_status(&ctx, CoinId::Verium)
            .await
            .expect("offline should map to Ok(disconnected), not Err");
        assert!(!status.connected);
        assert_eq!(status.state, "Offline");
    }

    #[test]
    fn coin_parsing_and_ports() {
        assert_eq!(CoinId::parse("vrm"), Some(CoinId::Verium));
        assert_eq!(CoinId::parse("Vericoin"), Some(CoinId::Vericoin));
        assert_eq!(CoinId::Verium.default_rpc_port(), 33987);
        assert_eq!(CoinId::Vericoin.default_rpc_port(), 33988);
    }
}
