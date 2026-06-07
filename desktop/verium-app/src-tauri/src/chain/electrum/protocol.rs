//! Electrum JSON-RPC line protocol (v1.4).

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct ElectrumRequest {
    pub id: u64,
    pub method: String,
    pub params: Value,
}

#[derive(Debug, Deserialize)]
pub struct ElectrumResponse {
    pub id: Option<u64>,
    pub result: Option<Value>,
    pub error: Option<ElectrumError>,
    pub method: Option<String>,
    pub params: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct ElectrumError {
    pub code: i32,
    pub message: String,
}

impl ElectrumError {
    pub fn is_rate_limited(&self) -> bool {
        self.code == -101
    }
}

impl ElectrumResponse {
    pub fn into_result(self) -> crate::error::AppResult<Value> {
        if let Some(err) = self.error {
            return Err(crate::error::AppError::Electrum {
                code: err.code,
                message: err.message,
            });
        }
        self.result
            .ok_or_else(|| crate::error::AppError::other("electrum response missing result"))
    }

    pub fn is_notification(&self) -> bool {
        self.id.is_none() && self.method.is_some()
    }
}
