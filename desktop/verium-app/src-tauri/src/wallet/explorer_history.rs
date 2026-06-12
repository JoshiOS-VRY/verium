//! Wallet transaction history via the Vericonomy explorer index.
//!
//! Per-address `netDelta` includes spent inputs (e.g. -280 VRM on the funding address).
//! Core `listtransactions` instead lists payment sends and change receives. We fetch each
//! indexed tx and expand it into the same row shape.

use std::collections::{HashMap, HashSet};

use futures_util::stream::{self, StreamExt};

use crate::chain::types::WalletTx;
use crate::coin_profile::CoinId;
use crate::error::AppResult;
use crate::indexer_api::{
    fetch_indexer_address, fetch_indexer_transaction, IndexerAddressTransaction,
};
use crate::wallet::listtransactions_rows::rows_from_indexer_transaction;

const MAX_ADDRESSES: usize = 128;
const PER_ADDRESS_LIMIT: u32 = 150;
const ADDRESS_FETCH_CONCURRENCY: usize = 12;
const TX_DETAIL_FETCH_CONCURRENCY: usize = 12;

#[derive(Clone)]
struct TxRef {
    txid: String,
    time: u64,
}

fn tx_ref_from_indexer_row(tx: &IndexerAddressTransaction) -> Option<TxRef> {
    let txid = tx.txid.trim();
    if txid.is_empty() {
        return None;
    }
    Some(TxRef {
        txid: txid.to_string(),
        time: tx.time.unwrap_or(0),
    })
}

/// Fetch and expand indexed history for wallet-owned addresses.
pub async fn fetch_wallet_history_from_explorer(
    coin: CoinId,
    addresses: &[String],
    limit: usize,
    tip: Option<u32>,
) -> AppResult<Vec<WalletTx>> {
    if addresses.is_empty() || limit == 0 {
        return Ok(Vec::new());
    }

    let wallet_addresses: HashSet<String> = addresses
        .iter()
        .map(|a| a.trim().to_string())
        .filter(|a| !a.is_empty())
        .collect();

    let unique: Vec<String> = {
        let mut seen = HashMap::new();
        let mut list = Vec::new();
        for addr in addresses {
            let clean = addr.trim();
            if clean.is_empty() || seen.contains_key(clean) {
                continue;
            }
            seen.insert(clean.to_string(), ());
            list.push(clean.to_string());
            if list.len() >= MAX_ADDRESSES {
                break;
            }
        }
        list
    };

    let per_addr_limit = PER_ADDRESS_LIMIT.min(limit as u32);
    let results = stream::iter(unique)
        .map(|address| {
            let address = address.clone();
            async move {
                let detail = fetch_indexer_address(coin, &address, per_addr_limit, 0).await?;
                Ok(detail)
            }
        })
        .buffer_unordered(ADDRESS_FETCH_CONCURRENCY)
        .collect::<Vec<AppResult<_>>>()
        .await;

    let mut tx_refs: HashMap<String, TxRef> = HashMap::new();
    for result in results {
        match result {
            Ok(detail) if detail.found => {
                for tx in &detail.transactions {
                    if let Some(row) = tx_ref_from_indexer_row(tx) {
                        tx_refs
                            .entry(row.txid.clone())
                            .and_modify(|existing| {
                                if row.time > existing.time {
                                    existing.time = row.time;
                                }
                            })
                            .or_insert(row);
                    }
                }
            }
            Ok(_) => {}
            Err(e) => {
                tracing::debug!("explorer address history failed: {e}");
            }
        }
    }

    if tx_refs.is_empty() {
        return Ok(Vec::new());
    }

    let mut ordered: Vec<TxRef> = tx_refs.into_values().collect();
    ordered.sort_by(|a, b| b.time.cmp(&a.time).then_with(|| b.txid.cmp(&a.txid)));
    ordered.truncate(limit);

    let txids: Vec<String> = ordered.into_iter().map(|r| r.txid).collect();
    let details = stream::iter(txids)
        .map(|txid| {
            let txid = txid.clone();
            async move {
                let detail = fetch_indexer_transaction(coin, &txid).await?;
                Ok((txid, detail))
            }
        })
        .buffer_unordered(TX_DETAIL_FETCH_CONCURRENCY)
        .collect::<Vec<AppResult<(String, _)>>>()
        .await;

    let mut rows: Vec<WalletTx> = Vec::new();
    for result in details {
        match result {
            Ok((_, detail)) if detail.found => {
                rows.extend(rows_from_indexer_transaction(&detail, &wallet_addresses, tip));
            }
            Ok(_) => {}
            Err(e) => {
                tracing::debug!("explorer tx detail failed: {e}");
            }
        }
    }

    rows.sort_by(|a, b| {
        let ta = a.time.unwrap_or(0);
        let tb = b.time.unwrap_or(0);
        tb.cmp(&ta)
            .then_with(|| b.txid.cmp(&a.txid))
            .then_with(|| b.category.cmp(&a.category))
    });
    rows.truncate(limit);
    Ok(rows)
}
