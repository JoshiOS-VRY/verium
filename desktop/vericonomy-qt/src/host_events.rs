//! Forward host bridge events to Qt controllers.

use std::sync::{Mutex, OnceLock};

static LISTENER: OnceLock<Mutex<Option<Box<dyn Fn(&str, serde_json::Value) + Send + Sync>>>> =
    OnceLock::new();

fn listener_slot() -> &'static Mutex<Option<Box<dyn Fn(&str, serde_json::Value) + Send + Sync>>> {
    LISTENER.get_or_init(|| Mutex::new(None))
}

pub fn set_host_event_listener(
    listener: impl Fn(&str, serde_json::Value) + Send + Sync + 'static,
) {
    *listener_slot().lock().expect("host event listener lock") = Some(Box::new(listener));
}

pub fn dispatch_host_event(event: &str, payload: serde_json::Value) {
    if let Some(cb) = listener_slot().lock().expect("host event listener lock").as_ref() {
        cb(event, payload);
    }
}
