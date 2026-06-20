//! Address book (P3).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::coin::CoinId;
use crate::config;
use crate::error::{HostError, HostResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressBookEntry {
    pub label: String,
    pub address: String,
    pub category: String,
}

fn path(coin: CoinId) -> PathBuf {
    config::app_config_base().join(format!("addressbook-{}.json", coin.as_str()))
}

pub fn list(coin: CoinId) -> HostResult<Vec<AddressBookEntry>> {
    let p = path(coin);
    if !p.exists() {
        return Ok(vec![]);
    }
    let text = std::fs::read_to_string(p)?;
    Ok(serde_json::from_str(&text).unwrap_or_default())
}

pub fn upsert(coin: CoinId, entry: AddressBookEntry) -> HostResult<()> {
    let mut rows = list(coin)?;
    if let Some(i) = rows.iter().position(|e| e.address == entry.address) {
        rows[i] = entry;
    } else {
        rows.push(entry);
    }
    save(coin, &rows)
}

pub fn delete(coin: CoinId, address: &str) -> HostResult<()> {
    let mut rows = list(coin)?;
    rows.retain(|e| e.address != address);
    save(coin, &rows)
}

fn save(coin: CoinId, rows: &[AddressBookEntry]) -> HostResult<()> {
    let p = path(coin);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(p, serde_json::to_string_pretty(rows)?)?;
    Ok(())
}

pub async fn import_bootstrap(_ctx: &crate::context::AppContext, _coin: CoinId, _path: &str) -> HostResult<()> {
    Err(HostError::other("Bootstrap import not yet ported (P1)"))
}
