//! Process-wide `AppContext` shared by every bridge controller.
//!
//! Built once at startup with the Qt `HostBridge` implementation, then handed
//! (as `Arc` clones) to controllers so their async work can reach the host layer
//! and the shared SDK. Mirrors how the Tauri app `manage`d a single `AppState`.

use std::sync::{Arc, OnceLock};

use vericonomy_desktop_host::{AppContext, DesktopHostBridge};

static CONTEXT: OnceLock<Arc<AppContext>> = OnceLock::new();

/// Get (initialising on first use) the shared application context.
pub fn context() -> Arc<AppContext> {
    CONTEXT
        .get_or_init(|| {
            // Desktop bridge: open_url/notify use OS handlers. The emit callback
            // (Qt signal hub) is installed in a later step; events are unused by
            // QML today since controllers poll + emit their own signals.
            let ctx = AppContext::new(Arc::new(DesktopHostBridge::new()));
            ctx.load_endpoints_from_conf();
            ctx
        })
        .clone()
}
