use std::collections::HashMap;
use std::hash::Hash;

/// Who owns the RPC port for a coin, based on the listening PIDs we observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortOwner {
    /// No process is listening on the port.
    None,
    /// A listener whose PID matches the daemon we spawned.
    Ours,
    /// A listener we did not spawn — must not be killed without user consent.
    Foreign,
}

impl PortOwner {
    /// True when the port is held by the daemon we spawned.
    pub fn is_ours(self) -> bool {
        matches!(self, PortOwner::Ours)
    }
}

/// Tracks the daemon we spawned per coin and the RPC port it owns, so the app
/// can distinguish "our node" from a foreign process on the same port. This is
/// the single authority for the process-deduplication policy: foreign listeners
/// are never killed implicitly.
#[derive(Debug, Default)]
pub struct ProcessRegistry<K: Eq + Hash + Copy> {
    managed_pid: HashMap<K, u32>,
    rpc_port: HashMap<K, u16>,
}

impl<K: Eq + Hash + Copy> ProcessRegistry<K> {
    pub fn new() -> Self {
        Self {
            managed_pid: HashMap::new(),
            rpc_port: HashMap::new(),
        }
    }

    /// Record a daemon we just spawned for `key` listening on `port`.
    pub fn record_spawn(&mut self, key: K, pid: u32, port: u16) {
        self.managed_pid.insert(key, pid);
        self.rpc_port.insert(key, port);
    }

    /// Forget the managed process for `key` (after a confirmed stop/exit).
    pub fn clear(&mut self, key: K) {
        self.managed_pid.remove(&key);
        self.rpc_port.remove(&key);
    }

    pub fn managed_pid(&self, key: K) -> Option<u32> {
        self.managed_pid.get(&key).copied()
    }

    pub fn rpc_port(&self, key: K) -> Option<u16> {
        self.rpc_port.get(&key).copied()
    }

    pub fn is_managed_pid(&self, key: K, pid: u32) -> bool {
        self.managed_pid.get(&key) == Some(&pid)
    }

    /// Classify the owner of the RPC port given the PIDs currently listening.
    pub fn classify_port_owner(&self, key: K, listening_pids: &[u32]) -> PortOwner {
        if listening_pids.is_empty() {
            return PortOwner::None;
        }
        match self.managed_pid.get(&key) {
            Some(pid) if listening_pids.contains(pid) => PortOwner::Ours,
            _ => PortOwner::Foreign,
        }
    }

    /// True only when it is safe to stop the listeners automatically: every
    /// listening PID is the daemon we manage for `key`.
    pub fn safe_to_kill(&self, key: K, listening_pids: &[u32]) -> bool {
        !listening_pids.is_empty()
            && self
                .managed_pid
                .get(&key)
                .is_some_and(|pid| listening_pids.iter().all(|p| p == pid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    enum Coin {
        Verium,
        Vericoin,
    }

    #[test]
    fn classifies_our_process() {
        let mut reg = ProcessRegistry::new();
        reg.record_spawn(Coin::Verium, 1234, 41988);
        assert_eq!(
            reg.classify_port_owner(Coin::Verium, &[1234]),
            PortOwner::Ours
        );
        assert!(reg.is_managed_pid(Coin::Verium, 1234));
    }

    #[test]
    fn classifies_foreign_process() {
        let mut reg = ProcessRegistry::new();
        reg.record_spawn(Coin::Verium, 1234, 41988);
        // A different PID owns the port -> foreign, never auto-killed.
        assert_eq!(
            reg.classify_port_owner(Coin::Verium, &[9999]),
            PortOwner::Foreign
        );
        assert!(!reg.safe_to_kill(Coin::Verium, &[9999]));
    }

    #[test]
    fn unknown_coin_with_listener_is_foreign() {
        let reg: ProcessRegistry<Coin> = ProcessRegistry::new();
        assert_eq!(
            reg.classify_port_owner(Coin::Vericoin, &[42]),
            PortOwner::Foreign
        );
        assert!(!reg.safe_to_kill(Coin::Vericoin, &[42]));
    }

    #[test]
    fn empty_listeners_is_none() {
        let reg: ProcessRegistry<Coin> = ProcessRegistry::new();
        assert_eq!(reg.classify_port_owner(Coin::Verium, &[]), PortOwner::None);
    }

    #[test]
    fn safe_to_kill_only_when_all_pids_are_ours() {
        let mut reg = ProcessRegistry::new();
        reg.record_spawn(Coin::Verium, 1234, 41988);
        assert!(reg.safe_to_kill(Coin::Verium, &[1234]));
        // A foreign PID also on the port makes it unsafe.
        assert!(!reg.safe_to_kill(Coin::Verium, &[1234, 5678]));
    }

    #[test]
    fn clear_forgets_process() {
        let mut reg = ProcessRegistry::new();
        reg.record_spawn(Coin::Verium, 1234, 41988);
        reg.clear(Coin::Verium);
        assert_eq!(reg.managed_pid(Coin::Verium), None);
        assert_eq!(reg.rpc_port(Coin::Verium), None);
    }
}
