//! Pure, dependency-light core for managing local node daemons.
//!
//! This crate holds the parts of node supervision that are *decisions*, not
//! side effects: the lifecycle state machine, a process/port ownership
//! registry, and retry/backoff guards. Keeping them free of Tauri, tokio, and
//! OS process APIs makes the control logic unit-testable and reusable, and lets
//! the Tauri app shrink to the I/O glue (spawning, probing, RPC).
//!
//! The app supplies observed signals (process alive? RPC reachable? auth ok?)
//! and this crate maps them to an authoritative [`NodeState`] plus a suggested
//! [`RecoveryHint`]. Process deduplication and spawn cooldowns are expressed as
//! explicit policy here rather than scattered ad-hoc flags.

mod backoff;
mod registry;
mod state;

pub use backoff::RetryGuard;
pub use registry::{PortOwner, ProcessRegistry};
pub use state::{state_label, NodeSignals, NodeState, RecoveryHint};

/// Map observed runtime signals to an authoritative lifecycle state and an
/// optional recovery hint. This is the single FSM transition function — the app
/// must not infer lifecycle state through parallel heuristics.
pub fn transition(signals: &NodeSignals) -> (NodeState, Option<RecoveryHint>) {
    state::transition(signals)
}
