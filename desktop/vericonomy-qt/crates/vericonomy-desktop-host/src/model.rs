//! Serde DTOs returned by host commands.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeStatus {
    pub connected: bool,
    pub warming_up: bool,
    pub chain: String,
    pub blocks: i64,
    pub headers: i64,
    pub verification_progress: f64,
    pub initial_block_download: bool,
    pub connections: i64,
    pub state: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WalletInfo {
    pub balance: f64,
    pub unconfirmed_balance: f64,
    pub immature_balance: f64,
    pub locked: bool,
    pub missing: bool,
    pub txcount: i64,
}

impl WalletInfo {
    pub fn total(&self) -> f64 {
        self.balance + self.unconfirmed_balance + self.immature_balance
    }
}

/// Mirrors `TransactionItem` from `src/lib/rpc/client.ts`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRow {
    pub txid: String,
    pub category: String,
    pub amount: f64,
    pub fee: Option<f64>,
    pub confirmations: i64,
    pub address: Option<String>,
    pub time: i64,
    pub time_label: String,
    /// Locale-formatted timestamp for table "When" column (`—` when unknown).
    pub time_display: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalancePoint {
    pub t: i64,
    pub balance: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MiningInfo {
    pub blocks: i64,
    pub currentblocksize: i64,
    pub currentblocktx: i64,
    pub difficulty: f64,
    pub networkhashps: f64,
    pub hashrate: f64,
    pub pooledtx: i64,
    pub blocksperhour: f64,
    pub blocktime: f64,
    pub blockreward: f64,
    pub chain: String,
    pub warnings: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StakingInfo {
    pub enabled: bool,
    pub stake_weight: f64,
    pub netstakeweight: f64,
    pub expected_time: f64,
    pub searches_per_second: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PeerRow {
    pub addr: String,
    pub subver: String,
    pub inbound: bool,
    pub startingheight: i64,
    pub synced_headers: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogLine {
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EarnState {
    pub active: bool,
    pub threads: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExplorerStats {
    pub network_hash: Option<f64>,
    pub supply: Option<f64>,
    pub height: Option<u64>,
    pub difficulty: Option<f64>,
    pub price_usd: Option<f64>,
    pub volume_24h_usd: Option<f64>,
    pub block_reward: Option<f64>,
    pub stake_interest: Option<f64>,
    pub pooled_tx: Option<i64>,
    pub blocks_per_hour: Option<f64>,
    pub block_time_min: Option<f64>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSnapshot {
    pub is_light: bool,
    pub connected: bool,
    pub synced: bool,
    pub state_label: String,
    pub network_mode: String,
    pub activity_title: String,
    pub local_blocks: i64,
    pub sync_target: i64,
    pub behind: i64,
    pub verification_progress: f64,
    pub connections: i64,
    pub wallet: WalletInfo,
    pub explorer: ExplorerStats,
    pub mining_active: bool,
    pub miner_active: bool,
    pub local_hashrate: f64,
    pub blocks_found: i64,
    pub staking_active: bool,
    pub stake_weight: f64,
    pub net_stake_weight: f64,
    pub network_share_percent: f64,
    pub est_daily_vrm: f64,
    pub block_time_min: f64,
    pub network_hash_khm: f64,
    pub peer_status: String,
    pub mempool: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExplorerBlock {
    pub height: u64,
    pub hash: String,
    pub time: u64,
    pub n_tx: Option<u64>,
    pub miner_address: Option<String>,
    pub size: Option<u64>,
    pub difficulty: Option<String>,
    pub output_total: Option<String>,
    pub mint: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SetupStatus {
    pub setup_completed: bool,
    pub wallet_ready: bool,
    pub node_reachable: bool,
    pub has_full_node_wallet: bool,
    pub has_light_wallet: bool,
    pub is_light_mode: bool,
}
