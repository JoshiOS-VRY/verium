//! Process-wide `AppContext` shared by every bridge controller.

use std::sync::{Arc, OnceLock};

use vericonomy_desktop_host::{AppContext, DesktopHostBridge};

static CONTEXT: OnceLock<Arc<AppContext>> = OnceLock::new();

pub fn context() -> Arc<AppContext> {
    CONTEXT
        .get_or_init(|| {
            let bridge = DesktopHostBridge::with_emitter(|event, payload| {
                crate::host_events::dispatch_host_event(event, payload);
            });
            let ctx = AppContext::new(Arc::new(bridge));
            ctx.load_endpoints_from_conf();
            ctx
        })
        .clone()
}
