//! Encrypted receive-request history (Tauri-compatible on-disk format).

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::coin::CoinId;
use crate::config;
use crate::error::{HostError, HostResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiveRequest {
    pub id: String,
    pub created_at: i64,
    pub label: String,
    pub message: String,
    pub amount: Option<f64>,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ReceiveRequestFile {
    #[serde(default)]
    entries: Vec<ReceiveRequest>,
}

fn path(coin: CoinId) -> PathBuf {
    config::app_config_base().join(format!("receive-requests-{}.json", coin.as_str()))
}

fn parse_file(raw: serde_json::Value) -> ReceiveRequestFile {
    if raw.is_null() {
        return ReceiveRequestFile::default();
    }
    if let Ok(file) = serde_json::from_value::<ReceiveRequestFile>(raw.clone()) {
        if !file.entries.is_empty() || raw.get("entries").is_some() {
            return file;
        }
    }
    if let Ok(entries) = serde_json::from_value::<Vec<ReceiveRequest>>(raw) {
        return ReceiveRequestFile { entries };
    }
    ReceiveRequestFile::default()
}

fn load_file(coin: CoinId) -> HostResult<ReceiveRequestFile> {
    let p = path(coin);
    if !p.is_file() {
        return Ok(ReceiveRequestFile::default());
    }
    let raw = fs::read_to_string(p)?;
    Ok(parse_file(serde_json::from_str(&raw)?))
}

fn save_file(coin: CoinId, file: &ReceiveRequestFile) -> HostResult<()> {
    let p = path(coin);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(p, serde_json::to_string_pretty(file)?)?;
    Ok(())
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn list(coin: CoinId) -> HostResult<Vec<ReceiveRequest>> {
    Ok(load_file(coin)?.entries)
}

pub fn append(coin: CoinId, mut entry: ReceiveRequest) -> HostResult<ReceiveRequest> {
    entry.address = entry.address.trim().to_string();
    if entry.address.is_empty() {
        return Err(HostError::other("address must not be empty"));
    }
    entry.label = entry.label.trim().to_string();
    entry.message = entry.message.trim().to_string();

    let mut file = load_file(coin)?;
    let now = now_ts();
    if entry.id.is_empty() {
        entry.id = uuid::Uuid::new_v4().simple().to_string();
    }
    if entry.created_at <= 0 {
        entry.created_at = now;
    }

    file.entries.retain(|e| e.id != entry.id);
    file.entries.insert(0, entry.clone());
    save_file(coin, &file)?;
    Ok(entry)
}

pub fn delete_entry(coin: CoinId, id: &str) -> HostResult<()> {
    let mut file = load_file(coin)?;
    let before = file.entries.len();
    file.entries.retain(|e| e.id != id);
    if file.entries.len() == before {
        return Err(HostError::other("receive request not found"));
    }
    save_file(coin, &file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_legacy_array_format() {
        let raw = serde_json::json!([{
            "id": "abc",
            "created_at": 1,
            "label": "tip jar",
            "message": "thanks",
            "amount": 1.5,
            "address": "VRM123"
        }]);
        let file = parse_file(raw);
        assert_eq!(file.entries.len(), 1);
    }
}
