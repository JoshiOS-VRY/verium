//! The host abstraction that replaces Tauri's `AppHandle`.
//!
//! Command bodies in the Tauri app reach the shell via `AppHandle` for three
//! kinds of side effects: emitting events, opening OS resources, and showing
//! native dialogs/notifications. Those are the only true Tauri coupling points
//! (~14 files). We funnel them through `HostBridge` so the same command logic
//! runs unchanged under Qt (the Qt shell implements this trait) — or under
//! tests (a no-op implementation).

use std::sync::Arc;

/// Implemented by each shell (Qt today; tests use [`NullHostBridge`]).
pub trait HostBridge: Send + Sync {
    /// Push an event to the UI. Under Tauri this was `app.emit(event, payload)`;
    /// under Qt the implementation forwards to a Qt signal.
    fn emit(&self, event: &str, payload: serde_json::Value);

    /// Open a URL/path with the OS default handler (was `tauri_plugin_opener`).
    fn open_url(&self, url: &str);

    /// Show a native notification (was `tauri_plugin_notification`).
    fn notify(&self, title: &str, body: &str);
}

/// Shared handle type used throughout the host layer.
pub type SharedHostBridge = Arc<dyn HostBridge>;

/// No-op bridge for headless/test contexts.
#[derive(Default)]
pub struct NullHostBridge;

impl HostBridge for NullHostBridge {
    fn emit(&self, _event: &str, _payload: serde_json::Value) {}
    fn open_url(&self, _url: &str) {}
    fn notify(&self, _title: &str, _body: &str) {}
}

/// Concrete desktop bridge backed by OS primitives.
///
/// `emit` forwards to an optional callback installed by the Qt shell (which
/// turns it into a Qt signal). `open_url`/`notify` use the platform handlers.
#[derive(Default)]
pub struct DesktopHostBridge {
    emitter: Option<Box<dyn Fn(&str, serde_json::Value) + Send + Sync>>,
}

impl DesktopHostBridge {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_emitter(
        emitter: impl Fn(&str, serde_json::Value) + Send + Sync + 'static,
    ) -> Self {
        Self {
            emitter: Some(Box::new(emitter)),
        }
    }
}

impl HostBridge for DesktopHostBridge {
    fn emit(&self, event: &str, payload: serde_json::Value) {
        if let Some(cb) = &self.emitter {
            cb(event, payload);
        }
    }

    fn open_url(&self, url: &str) {
        if crate::os::is_safe_external(url) {
            crate::os::open_external(url);
        }
    }

    fn notify(&self, title: &str, body: &str) {
        // Minimal stand-in for tauri_plugin_notification; a native toast is
        // wired in Phase 5 packaging.
        eprintln!("[notify] {title}: {body}");
    }
}
