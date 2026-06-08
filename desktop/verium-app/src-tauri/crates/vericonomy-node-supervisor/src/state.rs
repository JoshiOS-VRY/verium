use serde::{Deserialize, Serialize};

/// Authoritative lifecycle state for a managed node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeState {
    Initializing,
    BinaryMissing,
    ConfigInvalid,
    Stopped,
    Starting,
    DatadirLocked,
    PortInUse,
    WarmingUp,
    Reindexing,
    ConnectedSyncing,
    ConnectedReady,
    SyncStalled,
    AuthMismatch,
    ChainCorrupt,
    Failed,
}

/// User-facing recovery action the UI may offer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryHint {
    RestartNode,
    RepairChain,
    BootstrapChain,
    InstallBinary,
    ResetCredentials,
    ChangeDatadir,
    QuitOtherInstance,
    ClearInvalidBlock,
}

/// Observed signals gathered by the app (process table, RPC probes, log
/// parsing). The FSM consumes these; it never performs I/O itself.
#[derive(Debug, Clone, Default)]
pub struct NodeSignals {
    pub binary_found: bool,
    pub binary_manageable: bool,
    pub config_valid: bool,
    /// A managed child process is alive (per the process registry).
    pub process_alive: bool,
    /// RPC answered a `getblockchaininfo`-style probe (warmup counts as true).
    pub rpc_reachable: bool,
    /// RPC port has a listener whose PID is not the managed child.
    pub foreign_port_listener: bool,
    /// RPC rejected our credentials.
    pub rpc_auth_failed: bool,
    pub warming_up: bool,
    pub reindexing: bool,
    pub initial_block_download: bool,
    pub sync_stalled: bool,
    pub chain_corrupt: bool,
    pub datadir_locked: bool,
}

pub fn transition(s: &NodeSignals) -> (NodeState, Option<RecoveryHint>) {
    if !s.config_valid {
        return (NodeState::ConfigInvalid, Some(RecoveryHint::ChangeDatadir));
    }
    if !s.binary_found {
        return (NodeState::BinaryMissing, Some(RecoveryHint::InstallBinary));
    }
    if !s.binary_manageable {
        return (NodeState::Failed, Some(RecoveryHint::InstallBinary));
    }

    // Reachability is the strongest positive signal: if RPC answers we are at
    // least warming up, regardless of process-table noise.
    if s.rpc_reachable {
        if s.chain_corrupt {
            return (NodeState::ChainCorrupt, Some(RecoveryHint::RepairChain));
        }
        if s.reindexing {
            return (NodeState::Reindexing, None);
        }
        if s.warming_up {
            return (NodeState::WarmingUp, None);
        }
        if s.sync_stalled {
            return (NodeState::SyncStalled, Some(RecoveryHint::RestartNode));
        }
        if s.initial_block_download {
            return (NodeState::ConnectedSyncing, None);
        }
        return (NodeState::ConnectedReady, None);
    }

    // RPC not reachable.
    if s.rpc_auth_failed {
        return (NodeState::AuthMismatch, Some(RecoveryHint::ResetCredentials));
    }
    if s.datadir_locked {
        return (NodeState::DatadirLocked, Some(RecoveryHint::QuitOtherInstance));
    }
    if s.foreign_port_listener && !s.process_alive {
        // Something else owns the RPC port. Never kill it implicitly — ask.
        return (NodeState::PortInUse, Some(RecoveryHint::QuitOtherInstance));
    }
    if s.chain_corrupt {
        return (NodeState::ChainCorrupt, Some(RecoveryHint::RepairChain));
    }
    if s.reindexing {
        return (NodeState::Reindexing, None);
    }
    if s.process_alive {
        return (NodeState::Starting, None);
    }
    (NodeState::Stopped, Some(RecoveryHint::RestartNode))
}

pub fn state_label(state: NodeState) -> &'static str {
    match state {
        NodeState::Initializing => "Initializing",
        NodeState::BinaryMissing => "Node software missing",
        NodeState::ConfigInvalid => "Configuration error",
        NodeState::Stopped => "Stopped",
        NodeState::Starting => "Starting node",
        NodeState::DatadirLocked => "Data directory in use",
        NodeState::PortInUse => "Port in use",
        NodeState::WarmingUp => "Loading blockchain",
        NodeState::Reindexing => "Repairing blockchain",
        NodeState::ConnectedSyncing => "Syncing",
        NodeState::ConnectedReady => "Ready",
        NodeState::SyncStalled => "Sync stalled",
        NodeState::AuthMismatch => "Connection error",
        NodeState::ChainCorrupt => "Blockchain needs repair",
        NodeState::Failed => "Node unavailable",
    }
}

impl NodeState {
    pub fn as_str(self) -> &'static str {
        match self {
            NodeState::Initializing => "initializing",
            NodeState::BinaryMissing => "binary_missing",
            NodeState::ConfigInvalid => "config_invalid",
            NodeState::Stopped => "stopped",
            NodeState::Starting => "starting",
            NodeState::DatadirLocked => "datadir_locked",
            NodeState::PortInUse => "port_in_use",
            NodeState::WarmingUp => "warming_up",
            NodeState::Reindexing => "reindexing",
            NodeState::ConnectedSyncing => "connected_syncing",
            NodeState::ConnectedReady => "connected_ready",
            NodeState::SyncStalled => "sync_stalled",
            NodeState::AuthMismatch => "auth_mismatch",
            NodeState::ChainCorrupt => "chain_corrupt",
            NodeState::Failed => "failed",
        }
    }

    /// True for states where the node can serve wallet RPCs.
    pub fn is_connected(self) -> bool {
        matches!(
            self,
            NodeState::ConnectedSyncing | NodeState::ConnectedReady | NodeState::SyncStalled
        )
    }
}

impl RecoveryHint {
    pub fn as_str(self) -> &'static str {
        match self {
            RecoveryHint::RestartNode => "restart_node",
            RecoveryHint::RepairChain => "repair_chain",
            RecoveryHint::BootstrapChain => "bootstrap_chain",
            RecoveryHint::InstallBinary => "install_binary",
            RecoveryHint::ResetCredentials => "reset_credentials",
            RecoveryHint::ChangeDatadir => "change_datadir",
            RecoveryHint::QuitOtherInstance => "quit_other_instance",
            RecoveryHint::ClearInvalidBlock => "clear_invalid_block",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn healthy() -> NodeSignals {
        NodeSignals {
            binary_found: true,
            binary_manageable: true,
            config_valid: true,
            process_alive: true,
            rpc_reachable: true,
            ..Default::default()
        }
    }

    #[test]
    fn reachable_and_synced_is_ready() {
        let (state, hint) = transition(&healthy());
        assert_eq!(state, NodeState::ConnectedReady);
        assert!(hint.is_none());
    }

    #[test]
    fn ibd_is_syncing() {
        let mut s = healthy();
        s.initial_block_download = true;
        assert_eq!(transition(&s).0, NodeState::ConnectedSyncing);
    }

    #[test]
    fn warmup_wins_over_ready() {
        let mut s = healthy();
        s.warming_up = true;
        assert_eq!(transition(&s).0, NodeState::WarmingUp);
    }

    #[test]
    fn auth_failure_when_unreachable() {
        let mut s = healthy();
        s.rpc_reachable = false;
        s.rpc_auth_failed = true;
        let (state, hint) = transition(&s);
        assert_eq!(state, NodeState::AuthMismatch);
        assert_eq!(hint, Some(RecoveryHint::ResetCredentials));
    }

    #[test]
    fn foreign_port_without_our_process_is_port_in_use() {
        let mut s = healthy();
        s.rpc_reachable = false;
        s.process_alive = false;
        s.foreign_port_listener = true;
        let (state, hint) = transition(&s);
        assert_eq!(state, NodeState::PortInUse);
        assert_eq!(hint, Some(RecoveryHint::QuitOtherInstance));
    }

    #[test]
    fn missing_binary_short_circuits() {
        let mut s = healthy();
        s.binary_found = false;
        assert_eq!(transition(&s).0, NodeState::BinaryMissing);
    }

    #[test]
    fn alive_but_unreachable_is_starting() {
        let mut s = healthy();
        s.rpc_reachable = false;
        assert_eq!(transition(&s).0, NodeState::Starting);
    }

    #[test]
    fn dead_and_unreachable_is_stopped() {
        let mut s = healthy();
        s.rpc_reachable = false;
        s.process_alive = false;
        assert_eq!(transition(&s).0, NodeState::Stopped);
    }
}
