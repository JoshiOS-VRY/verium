//! Minimal JSON-RPC client for the bundled `veriumd` / `vericoind` daemons.
//!
//! Port of the essentials of `desktop/verium-app/src-tauri/src/rpc.rs`: HTTP
//! Basic-auth to a loopback daemon, with the daemon-warmup (-28) case surfaced
//! as [`HostError::Warmup`].

use serde_json::{json, Value};

use crate::error::{HostError, HostResult};

/// Connection parameters for one coin's daemon RPC endpoint.
#[derive(Debug, Clone)]
pub struct RpcEndpoint {
    pub url: String,
    pub user: String,
    pub password: String,
}

impl RpcEndpoint {
    pub fn loopback(port: u16, user: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            url: format!("http://127.0.0.1:{port}"),
            user: user.into(),
            password: password.into(),
        }
    }
}

/// Thin RPC caller bound to a reqwest client + endpoint.
pub struct RpcClient<'a> {
    http: &'a reqwest::Client,
    endpoint: &'a RpcEndpoint,
}

impl<'a> RpcClient<'a> {
    pub fn new(http: &'a reqwest::Client, endpoint: &'a RpcEndpoint) -> Self {
        Self { http, endpoint }
    }

    /// Invoke an RPC method, returning the `result` field or a mapped error.
    pub async fn call(&self, method: &str, params: Value) -> HostResult<Value> {
        let body = json!({
            "jsonrpc": "1.0",
            "id": "verium-qt",
            "method": method,
            "params": params,
        });

        let resp = self
            .http
            .post(&self.endpoint.url)
            .basic_auth(&self.endpoint.user, Some(&self.endpoint.password))
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let payload: Value = resp.json().await.map_err(|e| {
            // A non-JSON body during warmup is common; treat as warmup.
            if status.as_u16() == 503 {
                HostError::Warmup
            } else {
                HostError::Http(e.to_string())
            }
        })?;

        if let Some(err) = payload.get("error").and_then(|e| e.as_object()) {
            let code = err.get("code").and_then(|c| c.as_i64()).unwrap_or(0);
            let message = err
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("rpc error")
                .to_string();
            if code == -28 {
                return Err(HostError::Warmup);
            }
            return Err(HostError::Rpc { code, message });
        }

        Ok(payload.get("result").cloned().unwrap_or(Value::Null))
    }
}
