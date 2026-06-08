use serde::{Deserialize, Serialize};

use crate::daemon::DaemonBinaryStatus;

// The lifecycle FSM vocabulary now lives in the dependency-light
// `vericonomy-node-supervisor` crate (with the explicit transition function and
// process-dedup policy). Re-export so existing `crate::node::state::NodeState`
// references and the UI snapshot below keep working unchanged.
pub use vericonomy_node_supervisor::{state_label, NodeState, RecoveryHint};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSnapshot {
    pub state: NodeState,
    pub recovery_hint: Option<RecoveryHint>,
    /// Primary user-facing status line.
    pub message: String,
    pub detail: Option<String>,
    pub progress: Option<f64>,
    pub blocks: Option<u64>,
    pub headers: Option<u64>,
    pub connections: Option<u64>,
    pub managed: bool,
    pub binary: BinaryInfo,
    pub updated_at: u64,
    // Legacy compatibility fields for existing UI hooks.
    pub connected: bool,
    pub warming_up: bool,
    pub chain_corrupt: bool,
    pub reindex_in_progress: bool,
    pub sync_stalled: bool,
    pub error: Option<String>,
    pub daemon_phase: Option<String>,
    pub chain: Option<String>,
    pub verification_progress: Option<f64>,
    pub initial_block_download: Option<bool>,
    pub reindex_header: Option<u64>,
    pub chain_repair_detail: Option<String>,
    pub sync_stall_detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryInfo {
    pub found: bool,
    pub path: Option<String>,
    pub manageable: bool,
    pub runtime: String,
    pub missing_hint: Option<String>,
}

impl From<&DaemonBinaryStatus> for BinaryInfo {
    fn from(s: &DaemonBinaryStatus) -> Self {
        Self {
            found: s.found,
            path: s.path.clone(),
            manageable: s.manageable,
            runtime: s.runtime.clone(),
            missing_hint: s.missing_hint.clone(),
        }
    }
}

