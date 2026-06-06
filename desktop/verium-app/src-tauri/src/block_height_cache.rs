//! Bounded cache for block-hash → height lookups used by `list_transactions`.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;
use serde_json::{json, Value};

use crate::rpc::RpcClient;

const MAX_ENTRIES: usize = 2_048;
const TTL: Duration = Duration::from_secs(6 * 3600);
const MAX_LOOKUPS_PER_REQUEST: usize = 48;

struct Entry {
    height: u64,
    at: Instant,
}

static CACHE: Lazy<Mutex<HashMap<String, Entry>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn get_cached(hash: &str) -> Option<u64> {
    let guard = CACHE.lock().ok()?;
    let entry = guard.get(hash)?;
    if entry.at.elapsed() > TTL {
        return None;
    }
    Some(entry.height)
}

fn put_cached(hash: String, height: u64) {
    let Ok(mut guard) = CACHE.lock() else {
        return;
    };
    guard.insert(
        hash,
        Entry {
            height,
            at: Instant::now(),
        },
    );
    if guard.len() <= MAX_ENTRIES {
        return;
    }
    while guard.len() > MAX_ENTRIES {
        let Some(oldest_key) = guard
            .iter()
            .min_by_key(|(_, entry)| entry.at)
            .map(|(k, _)| k.clone())
        else {
            break;
        };
        guard.remove(&oldest_key);
    }
}

/// Resolve block heights via `getblockheader` (lightweight) with a bounded cache.
pub async fn resolve_block_heights(
    client: &RpcClient,
    hashes: impl IntoIterator<Item = String>,
) -> HashMap<String, u64> {
    let mut out = HashMap::new();
    let mut lookups = 0usize;

    for hash in hashes {
        if out.contains_key(&hash) {
            continue;
        }
        if let Some(height) = get_cached(&hash) {
            out.insert(hash, height);
            continue;
        }
        if lookups >= MAX_LOOKUPS_PER_REQUEST {
            break;
        }
        lookups += 1;

        let Ok(header) = client
            .call::<Value>("getblockheader", json!([hash]))
            .await
        else {
            continue;
        };
        let Some(height) = header.get("height").and_then(Value::as_u64) else {
            continue;
        };
        put_cached(hash.clone(), height);
        out.insert(hash, height);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evicts_when_over_max() {
        let mut guard = CACHE.lock().unwrap();
        guard.clear();
        for i in 0..MAX_ENTRIES + 10 {
            guard.insert(
                format!("hash{i}"),
                Entry {
                    height: i as u64,
                    at: Instant::now(),
                },
            );
        }
        drop(guard);
        put_cached("overflow".into(), 99);
        let guard = CACHE.lock().unwrap();
        assert!(guard.len() <= MAX_ENTRIES);
    }
}
