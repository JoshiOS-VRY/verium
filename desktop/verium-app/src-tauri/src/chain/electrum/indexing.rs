//! Rate limits for initial light-wallet address indexing (Electrum gap scan).

/// Max scripthash RPC calls per `sync_light_wallet` indexing slice.
pub const RPC_BUDGET_PER_SYNC: u32 = 24;
/// Addresses queried per Electrum batch during indexing.
pub const SCRIPTS_PER_BATCH: u32 = 4;
/// Scripthash calls before a pacing pause (indexing mode).
pub const BURST_SIZE: usize = 3;
pub const BURST_PAUSE_MS: u64 = 1200;

/// Steady-state refresh (known funded scripts only).
pub const REFRESH_BURST_SIZE: usize = 6;
pub const REFRESH_BURST_PAUSE_MS: u64 = 250;
