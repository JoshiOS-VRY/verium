//! Address book — same on-disk format as Tauri (`addressbook-{coin}.json`).

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::coin::CoinId;
use crate::config;
use crate::error::{HostError, HostResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressBookEntry {
    pub id: String,
    pub address: String,
    pub label: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default = "default_category")]
    pub category: String,
    pub created_at: i64,
    pub updated_at: i64,
}

fn default_category() -> String {
    "send".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct AddressBookFile {
    #[serde(default)]
    entries: Vec<AddressBookEntry>,
}

fn book_path(coin: CoinId) -> PathBuf {
    config::app_config_base().join(format!("addressbook-{}.json", coin.as_str()))
}

fn parse_addressbook_file(raw: serde_json::Value) -> AddressBookFile {
    if raw.is_null() {
        return AddressBookFile::default();
    }
    if let Ok(file) = serde_json::from_value::<AddressBookFile>(raw.clone()) {
        if !file.entries.is_empty() || raw.get("entries").is_some() {
            return file;
        }
    }
    if let Ok(entries) = serde_json::from_value::<Vec<AddressBookEntry>>(raw) {
        return AddressBookFile { entries };
    }
    AddressBookFile::default()
}

fn load_file(coin: CoinId) -> HostResult<AddressBookFile> {
    let path = book_path(coin);
    if !path.is_file() {
        return Ok(AddressBookFile::default());
    }
    let raw = fs::read_to_string(&path)?;
    Ok(parse_addressbook_file(serde_json::from_str(&raw)?))
}

fn save_file(coin: CoinId, file: &AddressBookFile) -> HostResult<()> {
    let path = book_path(coin);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, serde_json::to_string_pretty(file)?)?;
    Ok(())
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn list(coin: CoinId) -> HostResult<Vec<AddressBookEntry>> {
    Ok(load_file(coin)?.entries)
}

pub fn upsert(coin: CoinId, mut entry: AddressBookEntry) -> HostResult<AddressBookEntry> {
    entry.address = entry.address.trim().to_string();
    if entry.address.is_empty() {
        return Err(HostError::other("address must not be empty"));
    }
    if entry.label.trim().is_empty() {
        let short = if entry.address.len() > 12 {
            format!(
                "{}…{}",
                &entry.address[..6],
                &entry.address[entry.address.len() - 4..]
            )
        } else {
            entry.address.clone()
        };
        entry.label = short;
    } else {
        entry.label = entry.label.trim().to_string();
    }
    entry.notes = entry.notes.trim().to_string();
    if entry.category.trim().is_empty() {
        entry.category = default_category();
    } else {
        entry.category = entry.category.trim().to_string();
    }

    let mut file = load_file(coin)?;
    let now = now_ts();
    if entry.id.is_empty() {
        entry.id = uuid::Uuid::new_v4().simple().to_string();
        entry.created_at = now;
    }
    entry.updated_at = now;

    if let Some(pos) = file.entries.iter().position(|e| e.id == entry.id) {
        file.entries[pos] = entry.clone();
    } else {
        file.entries.retain(|e| {
            !(e.address.eq_ignore_ascii_case(&entry.address) && e.category == entry.category)
        });
        file.entries.push(entry.clone());
    }
    save_file(coin, &file)?;
    Ok(entry)
}

pub fn delete(coin: CoinId, id: &str) -> HostResult<()> {
    let mut file = load_file(coin)?;
    let before = file.entries.len();
    file.entries.retain(|e| e.id != id);
    if file.entries.len() == before {
        return Err(HostError::other("address book entry not found"));
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
            "address": "VRC123",
            "label": "friend",
            "notes": "",
            "category": "receive",
            "created_at": 1,
            "updated_at": 2
        }]);
        let file = parse_addressbook_file(raw);
        assert_eq!(file.entries.len(), 1);
        assert_eq!(file.entries[0].label, "friend");
    }

    #[test]
    fn parse_wrapped_entries_format() {
        let raw = serde_json::json!({ "entries": [] });
        let file = parse_addressbook_file(raw);
        assert!(file.entries.is_empty());
    }
}
