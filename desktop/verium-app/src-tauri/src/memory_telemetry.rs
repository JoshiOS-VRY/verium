//! Lightweight memory and runtime diagnostics for development builds.

use std::sync::atomic::{AtomicU32, Ordering};

use serde::Serialize;
use sysinfo::{Pid, ProcessesToUpdate, System};

static BACKGROUND_TASKS: AtomicU32 = AtomicU32::new(0);
static NODE_STATE_LISTENERS: AtomicU32 = AtomicU32::new(0);

/// Increment when a long-lived background task starts; decrement on clean shutdown.
pub fn track_background_task_start() {
    BACKGROUND_TASKS.fetch_add(1, Ordering::Relaxed);
}

pub fn track_background_task_end() {
    BACKGROUND_TASKS.fetch_sub(1, Ordering::Relaxed);
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryDiagnostics {
    pub wallet_process_rss_bytes: u64,
    pub background_tasks: u32,
    pub node_state_listener_count: u32,
    pub build_profile: String,
}

#[tauri::command]
pub fn set_node_state_listener_count(count: u32) {
    NODE_STATE_LISTENERS.store(count, Ordering::Relaxed);
}

#[tauri::command]
pub fn get_memory_diagnostics() -> MemoryDiagnostics {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let pid = Pid::from_u32(std::process::id());
    let rss = sys
        .process(pid)
        .map(|p| p.memory())
        .unwrap_or(0);

    MemoryDiagnostics {
        wallet_process_rss_bytes: rss,
        background_tasks: BACKGROUND_TASKS.load(Ordering::Relaxed),
        node_state_listener_count: NODE_STATE_LISTENERS.load(Ordering::Relaxed),
        build_profile: if cfg!(debug_assertions) {
            "debug".into()
        } else {
            "release".into()
        },
    }
}
