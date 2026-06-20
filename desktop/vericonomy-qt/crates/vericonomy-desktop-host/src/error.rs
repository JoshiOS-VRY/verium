//! Error type for the desktop host layer.
//!
//! Mirrors the surface of the Tauri app's `AppError`
//! (`desktop/verium-app/src-tauri/src/error.rs`) so command bodies port over
//! with minimal changes during Phase 1b.

use thiserror::Error;

pub type HostResult<T> = Result<T, HostError>;

#[derive(Debug, Error)]
pub enum HostError {
    #[error("rpc error {code}: {message}")]
    Rpc { code: i64, message: String },

    #[error("http error: {0}")]
    Http(String),

    #[error("io error: {0}")]
    Io(String),

    #[error("serialization error: {0}")]
    Serde(String),

    #[error("daemon is still warming up")]
    Warmup,

    #[error("{0}")]
    Other(String),
}

impl HostError {
    pub fn other(msg: impl Into<String>) -> Self {
        HostError::Other(msg.into())
    }

    /// True while the node is still loading (RPC -28); the UI shows a warming
    /// state instead of an error, matching the Tauri behaviour.
    pub fn is_warmup(&self) -> bool {
        matches!(self, HostError::Warmup)
            || matches!(self, HostError::Rpc { code, .. } if *code == -28)
    }
}

impl From<reqwest::Error> for HostError {
    fn from(e: reqwest::Error) -> Self {
        HostError::Http(e.to_string())
    }
}

impl From<serde_json::Error> for HostError {
    fn from(e: serde_json::Error) -> Self {
        HostError::Serde(e.to_string())
    }
}

impl From<std::io::Error> for HostError {
    fn from(e: std::io::Error) -> Self {
        HostError::Io(e.to_string())
    }
}
