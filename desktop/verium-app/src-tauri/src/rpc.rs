use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Semaphore;

use crate::coin_profile::CoinId;
use crate::config::DaemonConfig;
use crate::error::{AppError, AppResult};
use crate::node::constants::{RPC_TIMEOUT, STATUS_RPC_TIMEOUT};
use crate::node::rpc_auth::{resolve_managed_auth_methods, RpcAuth};

/// Shared reqwest clients keyed by timeout (in milliseconds). Each `Client` owns a
/// connection pool and idle-connection reaper; building one per RPC call leaks those
/// resources over a long session, so we reuse a single client per distinct timeout.
static HTTP_CLIENTS: Lazy<Mutex<HashMap<u64, Client>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Max concurrent in-flight RPC requests per daemon endpoint (host:port).
///
/// `veriumd`/`vericoind` serve RPC with a bounded HTTP work queue (we configure
/// `-rpcworkqueue=256`, `-rpcthreads=16`). Many independent callers — status
/// pollers, heal loops, `wait_for_rpc`, wallet hooks — can otherwise burst past
/// the queue while the node is still warming up, producing a flood of
/// "http work queue depth exceeded" rejections and a node that never reports
/// ready. Capping in-flight requests well below the queue depth makes callers
/// wait on this semaphore instead of overwhelming the daemon.
const MAX_INFLIGHT_PER_ENDPOINT: usize = 12;

static RPC_GATES: Lazy<Mutex<HashMap<String, Arc<Semaphore>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Cumulative count of logical RPC calls issued by the backend (one per
/// `RpcClient::call`, not per auth retry). Used by `get_memory_diagnostics`
/// to measure idle/active RPC volume for benchmarking. Cheap relaxed atomic.
static RPC_CALL_COUNT: AtomicU64 = AtomicU64::new(0);

/// Total RPC calls issued since process start.
pub fn rpc_call_count() -> u64 {
    RPC_CALL_COUNT.load(Ordering::Relaxed)
}

fn endpoint_gate(url: &str) -> Arc<Semaphore> {
    let mut gates = RPC_GATES.lock().expect("RPC gate map poisoned");
    gates
        .entry(url.to_string())
        .or_insert_with(|| Arc::new(Semaphore::new(MAX_INFLIGHT_PER_ENDPOINT)))
        .clone()
}

fn shared_http_client(timeout: Duration) -> AppResult<Client> {
    let key = timeout.as_millis() as u64;
    let mut clients = HTTP_CLIENTS
        .lock()
        .map_err(|_| AppError::other("HTTP client cache poisoned"))?;
    if let Some(client) = clients.get(&key) {
        return Ok(client.clone());
    }
    let client = Client::builder().timeout(timeout).build()?;
    clients.insert(key, client.clone());
    Ok(client)
}

#[derive(Debug, Clone)]
pub struct RpcClient {
    http: Client,
    url: String,
    auth_methods: Vec<RpcAuth>,
    gate: Arc<Semaphore>,
}

#[derive(Serialize)]
struct RpcRequest<'a> {
    jsonrpc: &'a str,
    id: &'a str,
    method: &'a str,
    params: Value,
}

#[derive(Deserialize)]
struct RpcResponse {
    result: Option<Value>,
    error: Option<RpcErrorBody>,
}

#[derive(Deserialize)]
struct RpcResponseWithId {
    #[serde(default)]
    id: Option<String>,
    result: Option<Value>,
    error: Option<RpcErrorBody>,
}

#[derive(Deserialize)]
struct RpcErrorBody {
    code: i64,
    message: String,
}

impl RpcClient {
    pub fn from_config(cfg: &DaemonConfig) -> AppResult<Self> {
        Self::from_config_for_coin(CoinId::Verium, cfg)
    }

    pub fn from_config_for_coin(coin: CoinId, cfg: &DaemonConfig) -> AppResult<Self> {
        Self::from_config_for_coin_with_timeout(coin, cfg, RPC_TIMEOUT)
    }

    pub fn status_client_for_coin(coin: CoinId, cfg: &DaemonConfig) -> AppResult<Self> {
        Self::from_config_for_coin_with_timeout(coin, cfg, STATUS_RPC_TIMEOUT)
    }

    pub fn from_config_with_timeout(cfg: &DaemonConfig, timeout: Duration) -> AppResult<Self> {
        Self::from_config_for_coin_with_timeout(CoinId::Verium, cfg, timeout)
    }

    pub fn from_config_for_coin_with_timeout(
        coin: CoinId,
        cfg: &DaemonConfig,
        timeout: Duration,
    ) -> AppResult<Self> {
        let url = format!("http://{}:{}/", cfg.rpc_host, cfg.rpc_port);
        let auth_methods = resolve_managed_auth_methods(coin, cfg)?;
        let http = shared_http_client(timeout)?;
        let gate = endpoint_gate(&url);
        Ok(Self {
            http,
            url,
            auth_methods,
            gate,
        })
    }

    pub async fn call<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: Value,
    ) -> AppResult<T> {
        RPC_CALL_COUNT.fetch_add(1, Ordering::Relaxed);
        // Bound concurrent requests to this endpoint so bursty callers wait here
        // rather than overflowing the daemon's HTTP work queue during warmup.
        let _permit = self
            .gate
            .acquire()
            .await
            .map_err(|_| AppError::other("RPC concurrency gate closed"))?;
        let mut last_unauthorized = None;
        for auth in &self.auth_methods {
            match self.call_with_auth(auth, method, params.clone()).await {
                Ok(v) => return Ok(v),
                Err(AppError::DaemonUnreachable(msg)) if msg.contains("unauthorized") => {
                    last_unauthorized = Some(msg);
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
        if self.auth_methods.is_empty() {
            return self.call_with_auth(&RpcAuth::None, method, params).await;
        }
        Err(AppError::DaemonUnreachable(
            last_unauthorized.unwrap_or_else(|| {
                "unauthorized: missing or invalid RPC credentials".into()
            }),
        ))
    }

    async fn call_with_auth<T: for<'de> Deserialize<'de>>(
        &self,
        auth: &RpcAuth,
        method: &str,
        params: Value,
    ) -> AppResult<T> {
        let body = RpcRequest {
            jsonrpc: "1.0",
            id: "vericonomy-app",
            method,
            params,
        };
        let req = self.http.post(&self.url).json(&body);
        let req = match auth {
            RpcAuth::UserPass(u, p) => req.basic_auth(u, Some(p)),
            RpcAuth::None => req,
        };
        let resp = req.send().await.map_err(|e| {
            if e.is_connect() || e.is_timeout() {
                AppError::DaemonUnreachable(e.to_string())
            } else {
                AppError::Http(e)
            }
        })?;
        let status = resp.status();
        let parsed: RpcResponse = if status.is_success() {
            resp.json().await?
        } else if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(AppError::DaemonUnreachable(
                "unauthorized: missing or invalid RPC credentials".into(),
            ));
        } else {
            let txt = resp.text().await.unwrap_or_default();
            match serde_json::from_str::<RpcResponse>(&txt) {
                Ok(p) => p,
                Err(_) => {
                    return Err(AppError::Other(format!("rpc http {status}: {txt}")))
                }
            }
        };
        if let Some(err) = parsed.error {
            return Err(AppError::Rpc {
                code: err.code,
                message: err.message,
            });
        }
        let result = parsed.result.unwrap_or(Value::Null);
        Ok(serde_json::from_value(result)?)
    }

    pub async fn call_no_result(&self, method: &str, params: Value) -> AppResult<()> {
        let _: Value = self.call(method, params).await?;
        Ok(())
    }

    /// Issue several JSON-RPC methods in a single HTTP request (JSON-RPC batch).
    /// `veriumd`/`vericoind` answer with an array of responses; results are
    /// returned in request order. Saves a round-trip versus sequential `call`s
    /// (e.g. status polling needs getblockchaininfo + getnetworkinfo together).
    pub async fn call_batch(&self, calls: &[(&str, Value)]) -> AppResult<Vec<AppResult<Value>>> {
        if calls.is_empty() {
            return Ok(Vec::new());
        }
        RPC_CALL_COUNT.fetch_add(1, Ordering::Relaxed);
        let _permit = self
            .gate
            .acquire()
            .await
            .map_err(|_| AppError::other("RPC concurrency gate closed"))?;

        let mut last_unauthorized = None;
        let auth_methods: Vec<&RpcAuth> = if self.auth_methods.is_empty() {
            vec![&RpcAuth::None]
        } else {
            self.auth_methods.iter().collect()
        };
        for auth in auth_methods {
            match self.call_batch_with_auth(auth, calls).await {
                Ok(v) => return Ok(v),
                Err(AppError::DaemonUnreachable(msg)) if msg.contains("unauthorized") => {
                    last_unauthorized = Some(msg);
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
        Err(AppError::DaemonUnreachable(last_unauthorized.unwrap_or_else(
            || "unauthorized: missing or invalid RPC credentials".into(),
        )))
    }

    async fn call_batch_with_auth(
        &self,
        auth: &RpcAuth,
        calls: &[(&str, Value)],
    ) -> AppResult<Vec<AppResult<Value>>> {
        let body: Vec<RpcRequest> = calls
            .iter()
            .enumerate()
            .map(|(i, (method, params))| RpcRequest {
                jsonrpc: "1.0",
                id: BATCH_IDS[i.min(BATCH_IDS.len() - 1)],
                method,
                params: params.clone(),
            })
            .collect();
        let req = self.http.post(&self.url).json(&body);
        let req = match auth {
            RpcAuth::UserPass(u, p) => req.basic_auth(u, Some(p)),
            RpcAuth::None => req,
        };
        let resp = req.send().await.map_err(|e| {
            if e.is_connect() || e.is_timeout() {
                AppError::DaemonUnreachable(e.to_string())
            } else {
                AppError::Http(e)
            }
        })?;
        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(AppError::DaemonUnreachable(
                "unauthorized: missing or invalid RPC credentials".into(),
            ));
        }
        if !status.is_success() {
            let txt = resp.text().await.unwrap_or_default();
            return Err(AppError::Other(format!("rpc http {status}: {txt}")));
        }
        // Responses may arrive out of order; map by id back to request order.
        let parsed: Vec<RpcResponseWithId> = resp.json().await?;
        let mut by_id: HashMap<String, AppResult<Value>> = HashMap::new();
        for r in parsed {
            let id = r.id.unwrap_or_default();
            let result = if let Some(err) = r.error {
                Err(AppError::Rpc {
                    code: err.code,
                    message: err.message,
                })
            } else {
                Ok(r.result.unwrap_or(Value::Null))
            };
            by_id.insert(id, result);
        }
        let mut out = Vec::with_capacity(calls.len());
        for i in 0..calls.len() {
            let id = BATCH_IDS[i.min(BATCH_IDS.len() - 1)];
            out.push(
                by_id
                    .remove(id)
                    .unwrap_or_else(|| Err(AppError::other("missing batch response"))),
            );
        }
        Ok(out)
    }
}

/// Stable string ids for batch positions (Bitcoin Core echoes the request id).
const BATCH_IDS: [&str; 8] = ["b0", "b1", "b2", "b3", "b4", "b5", "b6", "b7"];
