//! Indexer V2 JSON API (`/api/indexer/:chainId/...`) for in-app explorer detail screens.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::coin_profile::CoinId;
use crate::error::{AppError, AppResult};
use crate::explorer_api::EXPLORER_API_ENABLED;
use crate::http_shared::shared_http_client;
use crate::wallet::hd::{coins_to_sats, sats_to_coins};

const ADDRESS_PAGE_LIMIT: u32 = 100;
const ADDRESS_CUMULATIVE_MAX_TXS: u32 = 5000;

const HTTP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(12);
const HTTP_USER_AGENT: &str = "vericonomy-desktop-app/0.1";

/// Production explorer exposes indexer V2 query routes under `/v1/{chain}/…`
/// (not `/api/indexer/…`, which is only present in unreleased explorer builds).
fn indexer_api_url(coin: CoinId, path: &str) -> String {
    let base = coin.explorer_chain_api_base();
    let path = path.trim_start_matches('/');
    format!("{base}/{path}")
}

async fn get_json(url: &str) -> AppResult<Value> {
    let client = shared_http_client(HTTP_TIMEOUT, HTTP_USER_AGENT)?;
    let resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        return Err(AppError::other(format!(
            "indexer api {} returned http {}",
            url,
            resp.status()
        )));
    }
    Ok(resp.json().await?)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexerAmount {
    pub amount: String,
    pub ticker: String,
    #[serde(default)]
    pub decimal_places: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexerTransactionSummary {
    pub txid: String,
    #[serde(default)]
    pub block_height: Option<u64>,
    #[serde(default)]
    pub block_hash: Option<String>,
    #[serde(default)]
    pub tx_index: Option<u64>,
    #[serde(default)]
    pub time: Option<u64>,
    #[serde(default)]
    pub is_coinbase: bool,
    #[serde(default)]
    pub is_coinstake: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexerVin {
    pub n: u32,
    #[serde(default)]
    pub prev_txid: Option<String>,
    #[serde(default)]
    pub prev_vout: Option<u32>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub value: Option<IndexerAmount>,
    #[serde(default)]
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexerVout {
    pub n: u32,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub value: Option<IndexerAmount>,
    #[serde(default)]
    pub is_spent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexerAddressEvent {
    pub address: String,
    #[serde(default)]
    pub delta: Option<IndexerAmount>,
    #[serde(default)]
    pub event_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexerTransactionDetail {
    #[serde(default)]
    pub chain_id: Option<String>,
    /// Present on not-found responses; indexed hits only include txid under `transaction`.
    #[serde(default)]
    pub txid: String,
    pub found: bool,
    #[serde(default)]
    pub trusted: bool,
    #[serde(default)]
    pub transaction: Option<IndexerTransactionSummary>,
    #[serde(default)]
    pub inputs: Vec<IndexerVin>,
    #[serde(default)]
    pub outputs: Vec<IndexerVout>,
    #[serde(default)]
    pub address_events: Vec<IndexerAddressEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexerBlockSummary {
    pub height: u64,
    pub hash: String,
    #[serde(default)]
    pub previous_hash: Option<String>,
    #[serde(default)]
    pub next_hash: Option<String>,
    #[serde(default)]
    pub time: Option<u64>,
    #[serde(default)]
    pub tx_count: Option<u64>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub difficulty: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexerPaging {
    pub limit: u32,
    pub offset: u32,
    pub total: u64,
    #[serde(default)]
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexerBlockDetail {
    #[serde(default)]
    pub chain_id: Option<String>,
    pub found: bool,
    #[serde(default)]
    pub trusted: bool,
    #[serde(default)]
    pub block: Option<IndexerBlockSummary>,
    #[serde(default)]
    pub paging: Option<IndexerPaging>,
    #[serde(default)]
    pub transactions: Vec<IndexerTransactionSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexerAddressBalance {
    pub address: String,
    #[serde(default)]
    pub balance_atomic: Option<String>,
    #[serde(default)]
    pub balance: Option<IndexerAmount>,
    #[serde(default)]
    pub total_received_atomic: Option<String>,
    #[serde(default)]
    pub total_received: Option<IndexerAmount>,
    #[serde(default)]
    pub total_sent_atomic: Option<String>,
    #[serde(default)]
    pub total_sent: Option<IndexerAmount>,
    #[serde(default)]
    pub tx_count: Option<u64>,
    #[serde(default)]
    pub last_seen_height: Option<u64>,
    #[serde(default)]
    pub first_seen_height: Option<u64>,
    #[serde(default)]
    pub first_seen_time: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexerAddressRichlist {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub eligible: Option<bool>,
    #[serde(default)]
    pub rank: Option<u64>,
    #[serde(default)]
    pub total: Option<u64>,
    #[serde(default)]
    pub percentile: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexerAddressTransaction {
    pub txid: String,
    #[serde(default)]
    pub block_height: Option<u64>,
    #[serde(default)]
    pub block_hash: Option<String>,
    #[serde(default)]
    pub time: Option<u64>,
    #[serde(default)]
    pub net_delta_atomic: Option<String>,
    #[serde(default)]
    pub net_delta: Option<IndexerAmount>,
    #[serde(default)]
    pub is_coinbase: bool,
    #[serde(default)]
    pub is_coinstake: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexerAddressDetail {
    #[serde(default)]
    pub chain_id: Option<String>,
    pub address: String,
    pub found: bool,
    #[serde(default)]
    pub trusted: bool,
    #[serde(default)]
    pub balance: Option<IndexerAddressBalance>,
    #[serde(default)]
    pub richlist: Option<IndexerAddressRichlist>,
    #[serde(default)]
    pub paging: Option<IndexerPaging>,
    #[serde(default)]
    pub transactions: Vec<IndexerAddressTransaction>,
}

fn deserialize_indexer<T: serde::de::DeserializeOwned>(value: Value) -> AppResult<T> {
    serde_json::from_value(value).map_err(|e| AppError::other(format!("indexer parse error: {e}")))
}

pub async fn fetch_indexer_transaction(coin: CoinId, txid: &str) -> AppResult<IndexerTransactionDetail> {
    if !EXPLORER_API_ENABLED {
        return Err(AppError::other("explorer api disabled"));
    }
    let clean = txid.trim();
    if clean.is_empty() {
        return Err(AppError::other("empty txid"));
    }
    let url = indexer_api_url(coin, &format!("tx/{clean}"));
    let value = get_json(&url).await?;
    let mut detail: IndexerTransactionDetail = deserialize_indexer(value)?;
    if detail.txid.is_empty() {
        detail.txid = detail
            .transaction
            .as_ref()
            .map(|t| t.txid.clone())
            .filter(|id| !id.is_empty())
            .unwrap_or_else(|| clean.to_string());
    }
    Ok(detail)
}

pub async fn fetch_indexer_block(
    coin: CoinId,
    hash_or_height: &str,
    limit: u32,
    offset: u32,
) -> AppResult<IndexerBlockDetail> {
    if !EXPLORER_API_ENABLED {
        return Err(AppError::other("explorer api disabled"));
    }
    let query = format!("limit={limit}&offset={offset}");
    let value = hash_or_height.trim();
    if value.is_empty() {
        return Err(AppError::other("empty block id"));
    }
    let url = indexer_api_url(coin, &format!("block/{value}?{query}"));
    let json = get_json(&url).await?;
    deserialize_indexer(json)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CumulativeBalancePoint {
    #[serde(default)]
    pub time: Option<u64>,
    #[serde(default)]
    pub block_height: Option<u64>,
    pub balance: IndexerAmount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CumulativeBalanceSeries {
    pub points: Vec<CumulativeBalancePoint>,
    pub current_balance: IndexerAmount,
    pub tx_count_used: u64,
    #[serde(default)]
    pub tx_count_total: Option<u64>,
    pub complete: bool,
}

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

fn balance_sats(balance: &IndexerAddressBalance) -> i64 {
    if let Some(raw) = balance.balance_atomic.as_deref() {
        if let Ok(v) = raw.trim().parse::<i64>() {
            return v;
        }
    }
    if let Some(amount) = balance.balance.as_ref() {
        if let Ok(coins) = amount.amount.trim().parse::<f64>() {
            return coins_to_sats(coins);
        }
    }
    0
}

fn amount_from_sats(sats: i64, ticker: &str) -> IndexerAmount {
    IndexerAmount {
        amount: format!("{:.8}", sats_to_coins(sats)),
        ticker: ticker.to_string(),
        decimal_places: Some(8),
    }
}

fn tx_sort_key(tx: &IndexerAddressTransaction) -> (u64, u64, String) {
    (
        tx.time.unwrap_or(0),
        tx.block_height.unwrap_or(0),
        tx.txid.clone(),
    )
}

fn build_cumulative_forward(txs: &[IndexerAddressTransaction], ticker: &str) -> Vec<CumulativeBalancePoint> {
    let mut sorted: Vec<&IndexerAddressTransaction> = txs.iter().collect();
    sorted.sort_by_key(|tx| tx_sort_key(tx));

    let mut balance_sats = 0i64;
    let mut points = Vec::new();
    for tx in sorted {
        balance_sats += net_delta_sats(tx);
        points.push(CumulativeBalancePoint {
            time: tx.time,
            block_height: tx.block_height,
            balance: amount_from_sats(balance_sats, ticker),
        });
    }
    points
}

fn build_cumulative_backward(
    txs: &[IndexerAddressTransaction],
    anchor_sats: i64,
    ticker: &str,
) -> Vec<CumulativeBalancePoint> {
    let mut sorted: Vec<&IndexerAddressTransaction> = txs.iter().collect();
    sorted.sort_by(|a, b| tx_sort_key(b).cmp(&tx_sort_key(a)));

    let mut balance_sats = anchor_sats;
    let mut points = Vec::new();
    for tx in sorted {
        if tx.time.is_some() || tx.block_height.is_some() {
            points.push(CumulativeBalancePoint {
                time: tx.time,
                block_height: tx.block_height,
                balance: amount_from_sats(balance_sats, ticker),
            });
        }
        balance_sats -= net_delta_sats(tx);
    }

    points.sort_by_key(|p| (p.time.unwrap_or(0), p.block_height.unwrap_or(0)));
    points
}

async fn fetch_address_transactions_for_chart(
    coin: CoinId,
    address: &str,
    max_txs: u32,
) -> AppResult<(IndexerAddressDetail, Vec<IndexerAddressTransaction>, bool)> {
    let first = fetch_indexer_address(coin, address, ADDRESS_PAGE_LIMIT, 0).await?;
    let total = first
        .paging
        .as_ref()
        .map(|p| p.total)
        .unwrap_or(first.transactions.len() as u64);

    let mut all = first.transactions.clone();
    let mut offset = all.len() as u32;
    let cap = max_txs.min(ADDRESS_CUMULATIVE_MAX_TXS);

    while u64::from(offset) < total && (all.len() as u32) < cap {
        let limit = std::cmp::min(ADDRESS_PAGE_LIMIT, cap - all.len() as u32);
        let page = fetch_indexer_address(coin, address, limit, offset).await?;
        if page.transactions.is_empty() {
            break;
        }
        let fetched = page.transactions.len() as u32;
        all.extend(page.transactions);
        offset += fetched;
        let has_more = page.paging.as_ref().map(|p| p.has_more).unwrap_or(false);
        if !has_more && u64::from(offset) >= total {
            break;
        }
        if fetched < limit {
            break;
        }
    }

    let complete = all.len() as u64 >= total || u64::from(offset) >= total;
    Ok((first, all, complete))
}

pub async fn fetch_indexer_address_cumulative_series(
    coin: CoinId,
    address: &str,
    max_txs: Option<u32>,
) -> AppResult<CumulativeBalanceSeries> {
    if !EXPLORER_API_ENABLED {
        return Err(AppError::other("explorer api disabled"));
    }
    let clean = address.trim();
    if clean.is_empty() {
        return Err(AppError::other("empty address"));
    }

    let max_txs = max_txs.unwrap_or(ADDRESS_CUMULATIVE_MAX_TXS).min(ADDRESS_CUMULATIVE_MAX_TXS);
    let (detail, txs, complete) = fetch_address_transactions_for_chart(coin, clean, max_txs).await?;

    let balance_meta = detail.balance.as_ref();
    let ticker = balance_meta
        .and_then(|b| b.balance.as_ref())
        .map(|a| a.ticker.clone())
        .or_else(|| balance_meta.and_then(|b| b.total_received.as_ref()).map(|a| a.ticker.clone()))
        .unwrap_or_else(|| coin.symbol().to_string());

    let anchor_sats = balance_meta.map(balance_sats).unwrap_or(0);
    let current_balance = amount_from_sats(anchor_sats, &ticker);

    let points = if complete {
        build_cumulative_forward(&txs, &ticker)
    } else {
        build_cumulative_backward(&txs, anchor_sats, &ticker)
    };

    let tx_count_total = detail
        .paging
        .as_ref()
        .map(|p| p.total)
        .or_else(|| balance_meta.and_then(|b| b.tx_count));

    Ok(CumulativeBalanceSeries {
        points,
        current_balance,
        tx_count_used: txs.len() as u64,
        tx_count_total,
        complete,
    })
}

pub async fn fetch_indexer_address(
    coin: CoinId,
    address: &str,
    limit: u32,
    offset: u32,
) -> AppResult<IndexerAddressDetail> {
    if !EXPLORER_API_ENABLED {
        return Err(AppError::other("explorer api disabled"));
    }
    let clean = address.trim();
    if clean.is_empty() {
        return Err(AppError::other("empty address"));
    }
    let query = format!("limit={limit}&offset={offset}");
    let url = indexer_api_url(coin, &format!("address/{clean}?{query}"));
    let json = get_json(&url).await?;
    deserialize_indexer(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cumulative_forward_matches_anchor_balance() {
        let txs = vec![
            IndexerAddressTransaction {
                txid: "a".into(),
                block_height: Some(100),
                block_hash: None,
                time: Some(1_000),
                net_delta_atomic: Some("100000000".into()),
                net_delta: None,
                is_coinbase: false,
                is_coinstake: false,
            },
            IndexerAddressTransaction {
                txid: "b".into(),
                block_height: Some(200),
                block_hash: None,
                time: Some(2_000),
                net_delta_atomic: Some("-50000000".into()),
                net_delta: None,
                is_coinbase: false,
                is_coinstake: false,
            },
        ];
        let points = build_cumulative_forward(&txs, "VRM");
        assert_eq!(points.len(), 2);
        assert_eq!(points[0].balance.amount, "1.00000000");
        assert_eq!(points[1].balance.amount, "0.50000000");
    }

    #[test]
    fn parse_indexed_transaction_without_top_level_txid() {
        let json = serde_json::json!({
            "chainId": "vrm",
            "found": true,
            "trusted": true,
            "transaction": {
                "txid": "b737d0da57fedea8bfc564fc16b1cd173874aa725fbef54977e5de4f66b5826d",
                "blockHeight": 1102877,
                "time": 1781229077,
                "isCoinbase": true,
                "isCoinstake": false
            },
            "inputs": [{ "n": 0, "resolved": false }],
            "outputs": [{
                "n": 0,
                "address": "VJvLv7KD9AYod3jFXGGisw1QeBWVeq72Q8",
                "value": { "amount": "1.0", "ticker": "VRM", "decimalPlaces": 8 },
                "isSpent": false
            }],
            "addressEvents": []
        });
        let detail: IndexerTransactionDetail = deserialize_indexer(json).expect("parse");
        assert!(detail.found);
        assert_eq!(detail.txid, "");
        assert_eq!(
            detail.transaction.as_ref().map(|t| t.txid.as_str()),
            Some("b737d0da57fedea8bfc564fc16b1cd173874aa725fbef54977e5de4f66b5826d")
        );
    }
}
