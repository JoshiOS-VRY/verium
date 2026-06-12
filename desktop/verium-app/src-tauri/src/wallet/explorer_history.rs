//! Wallet transaction history via the Vericonomy explorer index (per-address API).
//!
//! Light wallets know which addresses belong to the user; the explorer index already
//! stores amount, time, and block height per address — no per-tx Electrum raw fetch.

use std::collections::HashMap;

use futures_util::stream::{self, StreamExt};

use crate::chain::types::WalletTx;
use crate::coin_profile::CoinId;
use crate::error::AppResult;
use crate::indexer_api::{
    fetch_indexer_address, IndexerAddressDetail, IndexerAddressTransaction,
};
use crate::wallet::hd::{coins_to_sats, sats_to_coins};

const MAX_ADDRESSES: usize = 32;
const PER_ADDRESS_LIMIT: u32 = 100;
const EXPLORER_FETCH_CONCURRENCY: usize = 4;

fn net_delta_sats(tx: &IndexerAddressTransaction) -> i64 {
    if let Some(raw) = tx.net_delta_atomic.as_deref() {
        if let Ok(v) = raw.trim().parse::<i64>() {
            return v;
        }
    }
    if let Some(amount) = tx.net_delta.as_ref() {
        if let Ok(coins) = amount.amount.trim().parse::<f64>() {
            return coins_to_sats(coins);
        }
    }
    0
}

fn category_for_delta_sats(delta_sats: i64, height: i32) -> String {
    if height <= 0 {
        if delta_sats < 0 {
            "send".into()
        } else {
            "unconfirmed".into()
        }
    } else if delta_sats < 0 {
        "send".into()
    } else {
        "receive".into()
    }
}

fn confirmations_for(tip: Option<u32>, block_height: Option<u64>) -> i32 {
    match (tip, block_height) {
        (Some(tip), Some(bh)) if tip >= bh as u32 => (tip - bh as u32 + 1) as i32,
        (Some(_), Some(_)) => 0,
        _ => 0,
    }
}

fn merge_indexer_tx(
    map: &mut HashMap<String, WalletTx>,
    tx: &IndexerAddressTransaction,
    address: &str,
    tip: Option<u32>,
) {
    let delta_sats = net_delta_sats(tx);
    let height = tx.block_height.unwrap_or(0) as i32;
    let blockheight = tx.block_height.map(|h| h as u32);
    let conf = confirmations_for(tip, tx.block_height);

    if let Some(existing) = map.get_mut(&tx.txid) {
        let merged_sats = coins_to_sats(existing.amount) + delta_sats;
        existing.amount = sats_to_coins(merged_sats);
        existing.category = category_for_delta_sats(merged_sats, existing.height);
        if tx.time.unwrap_or(0) > existing.time.unwrap_or(0) {
            existing.time = tx.time;
            existing.height = height;
            existing.blockheight = blockheight;
            existing.confirmations = conf;
            if let Some(hash) = &tx.block_hash {
                existing.blockhash = Some(hash.clone());
            }
        }
        if existing.address.is_none() {
            existing.address = Some(address.to_string());
        }
        return;
    }

    map.insert(
        tx.txid.clone(),
        WalletTx {
            txid: tx.txid.clone(),
            height,
            fee_sats: None,
            category: category_for_delta_sats(delta_sats, height),
            amount: sats_to_coins(delta_sats),
            address: Some(address.to_string()),
            confirmations: conf,
            time: tx.time,
            blockhash: tx.block_hash.clone(),
            blockheight,
        },
    );
}

/// Fetch and merge indexed history for wallet-owned addresses.
pub async fn fetch_wallet_history_from_explorer(
    coin: CoinId,
    addresses: &[String],
    limit: usize,
    tip: Option<u32>,
) -> AppResult<Vec<WalletTx>> {
    if addresses.is_empty() || limit == 0 {
        return Ok(Vec::new());
    }

    let unique: Vec<String> = {
        let mut seen = HashMap::new();
        for addr in addresses {
            let clean = addr.trim();
            if !clean.is_empty() {
                seen.insert(clean.to_string(), ());
            }
        }
        let mut list: Vec<String> = seen.keys().cloned().collect();
        list.sort();
        list.truncate(MAX_ADDRESSES);
        list
    };

    let per_addr_limit = PER_ADDRESS_LIMIT.min(limit as u32);
    let results = stream::iter(unique)
        .map(|address| {
            let address = address.clone();
            async move {
                let detail =
                    fetch_indexer_address(coin, &address, per_addr_limit, 0).await?;
                Ok((address, detail))
            }
        })
        .buffer_unordered(EXPLORER_FETCH_CONCURRENCY)
        .collect::<Vec<AppResult<(String, IndexerAddressDetail)>>>()
        .await;

    let mut merged: HashMap<String, WalletTx> = HashMap::new();
    for result in results {
        match result {
            Ok((address, detail)) if detail.found => {
                for tx in &detail.transactions {
                    merge_indexer_tx(&mut merged, tx, &address, tip);
                }
            }
            Ok(_) => {}
            Err(e) => {
                tracing::debug!("explorer address history failed: {e}");
            }
        }
    }

    let mut rows: Vec<WalletTx> = merged.into_values().collect();
    rows.sort_by(|a, b| {
        let ta = a.time.unwrap_or(0);
        let tb = b.time.unwrap_or(0);
        tb.cmp(&ta).then_with(|| b.txid.cmp(&a.txid))
    });
    rows.truncate(limit);
    Ok(rows)
}
