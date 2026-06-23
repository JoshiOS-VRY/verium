//! Tauri-free command functions.
//!
//! Each was a `#[tauri::command]` taking `State<AppState>` + `AppHandle`; here
//! they are plain `async fn(ctx: &AppContext, ...) -> HostResult<T>`. The Qt
//! bridge controllers (Phase 2) call these; during migration the Tauri app can
//! also delegate to them so it stays shippable (Phase 1b).

pub mod addressbook;
pub mod bootstrap;
pub mod daemon;
pub mod dashboard;
pub mod diagnostics;
pub mod explorer;
pub mod light_wallet;
pub mod mining;
pub mod network;
pub mod node;
pub mod pool;
pub mod pool_stats;
pub mod setup;
pub mod security;
pub mod staking;
pub mod transactions;
pub mod wallet;
pub mod wallet_mode;
