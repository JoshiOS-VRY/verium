//! Shared tokio runtime for the Qt bridge.
//!
//! Mirrors the pattern used by the iOS UniFFI layer
//! (`vericonomy-sdk/vericonomy-ffi/src/lib.rs`): a single multi-threaded runtime
//! that all bridge controllers spawn async work onto, marshalling results back
//! onto the Qt thread via `CxxQtThread::queue`.

use std::sync::OnceLock;
use tokio::runtime::Runtime;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("verium-qt-rt")
            .build()
            .expect("failed to build tokio runtime")
    })
}
