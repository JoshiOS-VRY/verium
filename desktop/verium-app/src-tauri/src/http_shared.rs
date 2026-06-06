//! Shared reqwest clients for explorer/pool HTTP APIs.
//!
//! Each `Client` owns a connection pool and idle reaper; building one per call
//! leaks those resources over a long session.

use std::sync::Mutex;
use std::time::Duration;

use once_cell::sync::Lazy;
use reqwest::Client;

use crate::error::{AppError, AppResult};

static HTTP_CLIENTS: Lazy<Mutex<Vec<(Duration, Client)>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

pub fn shared_http_client(timeout: Duration, user_agent: &str) -> AppResult<Client> {
    let mut clients = HTTP_CLIENTS
        .lock()
        .map_err(|_| AppError::other("HTTP client cache poisoned"))?;
    if let Some((_, client)) = clients.iter().find(|(t, _)| *t == timeout) {
        return Ok(client.clone());
    }
    let client = Client::builder()
        .timeout(timeout)
        .user_agent(user_agent)
        .build()?;
    clients.push((timeout, client.clone()));
    Ok(client)
}
