use std::collections::HashMap;
use std::time::{Duration, Instant};

use futures_util::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::coin_profile::CoinId;
use crate::error::{AppError, AppResult};
use crate::http_shared::shared_http_client;

pub const EXPLORER_API_ENABLED: bool = true;

const CACHE_TTL: Duration = Duration::from_secs(30);
const BLOCKS_CACHE_TTL: Duration = Duration::from_secs(8);
const MAX_OUTPUT_DETAIL_ENRICH: usize = 24;
const PEERS_CACHE_TTL: Duration = Duration::from_secs(300);
const MAX_EXTRACTION_CACHE_KEYS: usize = 32;
const HTTP_TIMEOUT: Duration = Duration::from_secs(12);
const HTTP_USER_AGENT: &str = "vericonomy-desktop-app/0.1";

fn explorer_api_url(coin: CoinId, path: &str) -> String {
    let base = coin.explorer_api_base();
    let path = path.trim_start_matches('/');
    format!("{base}/{path}")
}

fn explorer_chain_api_url(coin: CoinId, path: &str) -> String {
    let base = coin.explorer_chain_api_base();
    let path = path.trim_start_matches('/');
    format!("{base}/{path}")
}

pub fn explorer_logo_url(coin: CoinId) -> String {
    coin.explorer_logo_url().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerStats {
    pub network_hash: Option<f64>,
    pub supply: Option<f64>,
    pub height: Option<u64>,
    pub block_reward: Option<f64>,
    pub difficulty: Option<f64>,
    pub blocks_per_hour: Option<f64>,
    pub block_time_min: Option<f64>,
    pub pooled_tx: Option<u64>,
    pub price_usd: Option<f64>,
    pub price_btc: Option<f64>,
    pub market_cap_usd: Option<f64>,
    pub volume_24h_usd: Option<f64>,
    /// Vericoin PoS fields from `getmininginfo` (local node or explorer RPC proxy).
    pub stake_interest: Option<f64>,
    pub stake_inflation: Option<f64>,
    pub net_stake_weight: Option<f64>,
    pub pos_difficulty: Option<f64>,
    pub pow_difficulty: Option<f64>,
    pub fetched_at: u64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerChainTip {
    pub id: u64,
    pub height: u64,
    pub hash: String,
    pub branchlen: u64,
    pub status_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerBlock {
    pub id: u64,
    pub hash: String,
    pub height: u64,
    pub time: u64,
    pub mint: Option<String>,
    pub difficulty: Option<String>,
    pub n_tx: Option<u64>,
    pub miner_address: Option<String>,
    pub size: Option<u64>,
    pub output_total: Option<String>,
    pub output_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerTransaction {
    pub id: u64,
    pub txid: String,
    pub time: u64,
    pub fee: Option<String>,
    pub output_total: Option<String>,
    pub block_height: Option<u64>,
    pub block_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerExtractionEntry {
    pub rank: Option<String>,
    pub address: String,
    pub count: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerPeerEntry {
    pub id: u64,
    pub address: String,
    pub ip: String,
    pub port: u16,
    pub subversion: String,
    pub protocol_version: u64,
    pub connected_on_explorer: bool,
    pub last_seen: Option<String>,
}

struct TimedEntry<T> {
    at: Instant,
    value: T,
}

struct CoinCache {
    stats: Option<TimedEntry<ExplorerStats>>,
    blocks: Option<TimedEntry<Vec<ExplorerBlock>>>,
    transactions: Option<TimedEntry<Vec<ExplorerTransaction>>>,
    extractions: HashMap<String, TimedEntry<Vec<ExplorerExtractionEntry>>>,
    chain_tips: Option<TimedEntry<Vec<ExplorerChainTip>>>,
    peers: Option<TimedEntry<Vec<ExplorerPeerEntry>>>,
}

impl Default for CoinCache {
    fn default() -> Self {
        Self {
            stats: None,
            blocks: None,
            transactions: None,
            extractions: HashMap::new(),
            chain_tips: None,
            peers: None,
        }
    }
}

static CACHE: once_cell::sync::Lazy<Mutex<HashMap<CoinId, CoinCache>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

fn http_client() -> AppResult<reqwest::Client> {
    shared_http_client(HTTP_TIMEOUT, HTTP_USER_AGENT)
}

fn evict_stale_extractions(map: &mut HashMap<String, TimedEntry<Vec<ExplorerExtractionEntry>>>) {
    map.retain(|_, entry| entry.at.elapsed() < CACHE_TTL);
    while map.len() > MAX_EXTRACTION_CACHE_KEYS {
        if let Some(oldest_key) = map
            .iter()
            .min_by_key(|(_, entry)| entry.at)
            .map(|(k, _)| k.clone())
        {
            map.remove(&oldest_key);
        } else {
            break;
        }
    }
}

async fn get_json(client: &reqwest::Client, url: &str) -> AppResult<Value> {
    let resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        return Err(AppError::other(format!(
            "explorer api {} returned http {}",
            url,
            resp.status()
        )));
    }
    Ok(resp.json().await?)
}

fn parse_u64(v: &Value) -> Option<u64> {
    match v {
        Value::Number(n) => n.as_u64(),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

pub async fn fetch_network_stats(coin: CoinId) -> AppResult<ExplorerStats> {
    if !EXPLORER_API_ENABLED {
        return Err(AppError::other("explorer api disabled"));
    }

    if let Some(cached) = read_stats_cache(coin).await {
        return Ok(cached);
    }

    // The explorer-v2 `/wallet/stats` compat route aggregates the network,
    // market, and mining RPC fields into the legacy `ExplorerStats` shape.
    let client = http_client()?;
    let value = get_json(&client, &explorer_api_url(coin, "stats")).await?;
    let stats: ExplorerStats = serde_json::from_value(value)?;

    write_stats_cache(coin, stats.clone()).await;
    Ok(stats)
}

pub async fn fetch_explorer_peers(coin: CoinId) -> AppResult<Vec<ExplorerPeerEntry>> {
    if !EXPLORER_API_ENABLED {
        return Err(AppError::other("explorer api disabled"));
    }

    if let Some(cached) = read_peers_cache(coin).await {
        return Ok(cached);
    }

    // The explorer-v2 `/wallet/peers` compat route returns a flat list already
    // mapped to the legacy `ExplorerPeerEntry` shape (deduped by the server).
    let client = http_client()?;
    let value = get_json(&client, &explorer_api_url(coin, "peers?limit=50")).await?;
    let peers: Vec<ExplorerPeerEntry> = serde_json::from_value(value)?;

    write_peers_cache(coin, peers.clone()).await;
    Ok(peers)
}

fn parse_latest_api_block(item: &Value) -> Option<ExplorerBlock> {
    let height = parse_u64(item.get("height")?)?;
    let hash = item.get("hash")?.as_str()?.to_string();
    let output_total = item
        .get("outputValue")
        .and_then(|v| v.as_str().map(str::to_string))
        .or_else(|| item.get("mint").and_then(|v| v.as_str().map(str::to_string)))
        .or_else(|| {
            item.get("outputTotal")
                .and_then(|v| v.as_str().map(str::to_string))
        });
    let miner_address = item
        .get("extractedByAddress")
        .and_then(|v| v.as_str().map(str::to_string))
        .or_else(|| {
            item.get("miner")
                .and_then(|m| m.get("address"))
                .and_then(|v| v.as_str().map(str::to_string))
        });
    Some(ExplorerBlock {
        id: height,
        hash,
        height,
        time: parse_u64(item.get("time")?).unwrap_or(0),
        mint: output_total.clone(),
        difficulty: item
            .get("difficulty")
            .and_then(|v| v.as_str().map(str::to_string)),
        n_tx: item
            .get("txCount")
            .and_then(parse_u64)
            .or_else(|| item.get("nTx").and_then(parse_u64)),
        miner_address,
        size: item.get("size").and_then(parse_u64),
        output_total,
        output_count: item
            .get("outputCount")
            .and_then(parse_u64)
            .or_else(|| item.get("outputC").and_then(parse_u64)),
    })
}

fn parse_block_detail_payload(value: &Value) -> Option<ExplorerBlock> {
    let block = value.get("block")?;
    let mut row = parse_latest_api_block(block)?;
    if row.output_total.is_none() {
        if let Some(amount) = value
            .get("coinbase")
            .and_then(|c| c.get("reward"))
            .and_then(|r| r.get("amount"))
            .and_then(|v| v.as_str())
        {
            row.output_total = Some(amount.to_string());
            row.mint = Some(amount.to_string());
        }
    }
    Some(row)
}

fn block_needs_output_enrichment(block: &ExplorerBlock) -> bool {
    block.output_total.is_none() && block.mint.is_none()
}

async fn enrich_blocks_output_from_detail(
    coin: CoinId,
    blocks: Vec<ExplorerBlock>,
) -> AppResult<Vec<ExplorerBlock>> {
    let heights: Vec<u64> = blocks
        .iter()
        .filter(|block| block_needs_output_enrichment(block))
        .map(|block| block.height)
        .take(MAX_OUTPUT_DETAIL_ENRICH)
        .collect();
    if heights.is_empty() {
        return Ok(blocks);
    }

    let enriched = fetch_explorer_blocks_for_feed(coin, heights).await?;
    let by_height: HashMap<u64, ExplorerBlock> =
        enriched.into_iter().map(|b| (b.height, b)).collect();

    Ok(blocks
        .into_iter()
        .map(|block| {
            if let Some(detail) = by_height.get(&block.height) {
                return ExplorerBlock {
                    output_total: detail.output_total.clone().or(block.output_total),
                    mint: detail.mint.clone().or(block.mint),
                    output_count: detail.output_count.or(block.output_count),
                    miner_address: detail.miner_address.clone().or(block.miner_address),
                    size: detail.size.or(block.size),
                    difficulty: detail.difficulty.clone().or(block.difficulty),
                    n_tx: detail.n_tx.or(block.n_tx),
                    time: if block.time > 0 { block.time } else { detail.time },
                    hash: if block.hash.starts_with("light-tip-")
                        || block.hash.starts_with("local-pending-")
                    {
                        detail.hash.clone()
                    } else {
                        block.hash
                    },
                    ..block
                };
            }
            block
        })
        .collect())
}

async fn fetch_block_detail_row(
    client: &reqwest::Client,
    coin: CoinId,
    height: u64,
) -> Option<ExplorerBlock> {
    let url = explorer_chain_api_url(coin, &format!("block/{height}"));
    let value = get_json(client, &url).await.ok()?;
    if value.get("found").and_then(|v| v.as_bool()) == Some(false) {
        return None;
    }
    parse_block_detail_payload(&value)
}

/// Block rows for the recent-blocks table from explorer `/block/:height` (light wallet).
pub async fn fetch_explorer_blocks_for_feed(
    coin: CoinId,
    heights: Vec<u64>,
) -> AppResult<Vec<ExplorerBlock>> {
    if !EXPLORER_API_ENABLED {
        return Err(AppError::other("explorer api disabled"));
    }
    if heights.is_empty() {
        return Ok(Vec::new());
    }

    let client = http_client()?;
    let fetches = heights.into_iter().map(|height| {
        let client = client.clone();
        async move { fetch_block_detail_row(&client, coin, height).await }
    });
    let mut rows: Vec<ExplorerBlock> = join_all(fetches).await.into_iter().flatten().collect();
    rows.sort_by(|a, b| b.height.cmp(&a.height));
    Ok(rows)
}

pub async fn fetch_blocks(coin: CoinId, limit: u32) -> AppResult<Vec<ExplorerBlock>> {
    let limit = limit.clamp(1, 10);
    let blocks = if let Some(cached) = read_blocks_cache(coin).await {
        cached
    } else {
        let client = http_client()?;
        let url = explorer_chain_api_url(coin, "blocks/latest?limit=100");
        let value = get_json(&client, &url).await?;
        let arr = value
            .as_array()
            .ok_or_else(|| AppError::other("blocks/latest response is not an array"))?;

        arr.iter().filter_map(parse_latest_api_block).collect()
    };

    let blocks = enrich_blocks_output_from_detail(coin, blocks).await?;
    write_blocks_cache(coin, blocks.clone()).await;
    Ok(blocks.into_iter().take(limit as usize).collect())
}

pub async fn fetch_transactions(coin: CoinId, limit: u32) -> AppResult<Vec<ExplorerTransaction>> {
    let limit = limit.clamp(1, 100);
    if let Some(cached) = read_transactions_cache(coin).await {
        return Ok(cached.into_iter().take(limit as usize).collect());
    }

    let client = http_client()?;
    let url = explorer_api_url(coin, &format!("transactions?limit={limit}"));
    let value = get_json(&client, &url).await?;
    let arr = value
        .as_array()
        .ok_or_else(|| AppError::other("transactions response is not an array"))?;

    let txs: Vec<ExplorerTransaction> = arr
        .iter()
        .filter_map(|item| {
            let block = item.get("blocks").and_then(|b| b.as_array()).and_then(|a| a.first());
            Some(ExplorerTransaction {
                id: parse_u64(item.get("id")?)?,
                txid: item.get("txid")?.as_str()?.to_string(),
                time: parse_u64(item.get("time")?)?,
                fee: item.get("fee").and_then(|v| v.as_str().map(str::to_string)),
                output_total: item
                    .get("outputT")
                    .and_then(|v| v.as_str().map(str::to_string)),
                block_height: block.and_then(|b| b.get("height")).and_then(parse_u64),
                block_hash: block
                    .and_then(|b| b.get("hash"))
                    .and_then(|v| v.as_str().map(str::to_string)),
            })
        })
        .collect();

    write_transactions_cache(coin, txs.clone()).await;
    Ok(txs.into_iter().take(limit as usize).collect())
}

pub async fn fetch_extraction(
    coin: CoinId,
    limit: u32,
    period: &str,
) -> AppResult<Vec<ExplorerExtractionEntry>> {
    let limit = limit.clamp(1, 100);
    let period = normalize_extraction_period(period);
    let cache_key = format!("{period}:{limit}");
    if let Some(cached) = read_extraction_cache(coin, &cache_key).await {
        return Ok(cached.into_iter().take(limit as usize).collect());
    }

    let client = http_client()?;
    let url = explorer_api_url(
        coin,
        &format!("extraction?limit={limit}&period={period}"),
    );
    let value = get_json(&client, &url).await?;
    let arr = value
        .as_array()
        .ok_or_else(|| AppError::other("extraction response is not an array"))?;

    let entries: Vec<ExplorerExtractionEntry> = arr
        .iter()
        .filter_map(|item| {
            Some(ExplorerExtractionEntry {
                rank: item
                    .get("rank")
                    .and_then(|v| v.as_str().map(str::to_string)),
                address: item.get("address")?.as_str()?.to_string(),
                count: item
                    .get("count")
                    .and_then(|v| v.as_str().map(str::to_string)),
            })
        })
        .collect();

    write_extraction_cache(coin, &cache_key, entries.clone()).await;
    Ok(entries.into_iter().take(limit as usize).collect())
}

fn normalize_extraction_period(period: &str) -> &'static str {
    match period.trim().to_lowercase().as_str() {
        "week" | "7d" => "week",
        "month" | "30d" => "month",
        "year" => "year",
        "all" => "all",
        _ => "month",
    }
}

pub async fn fetch_chain_tips(coin: CoinId) -> AppResult<Vec<ExplorerChainTip>> {
    if let Some(cached) = read_chain_cache(coin).await {
        return Ok(cached);
    }

    let client = http_client()?;
    let url = explorer_api_url(coin, "chain");
    let value = get_json(&client, &url).await?;
    let arr = value
        .as_array()
        .ok_or_else(|| AppError::other("chain response is not an array"))?;

    let mut tips: Vec<ExplorerChainTip> = arr
        .iter()
        .filter_map(|item| {
            Some(ExplorerChainTip {
                id: parse_u64(item.get("id")?)?,
                height: parse_u64(item.get("height")?)?,
                hash: item.get("hash")?.as_str()?.to_string(),
                branchlen: parse_u64(item.get("branchlen")?).unwrap_or(0),
                status_name: item
                    .get("status")
                    .and_then(|s| s.get("name"))
                    .and_then(|v| v.as_str().map(str::to_string)),
            })
        })
        .collect();

    tips.sort_by(|a, b| b.height.cmp(&a.height).then(b.id.cmp(&a.id)));

    write_chain_cache(coin, tips.clone()).await;
    Ok(tips)
}

async fn with_coin_cache<F>(coin: CoinId, f: F)
where
    F: FnOnce(&mut CoinCache),
{
    let mut guard = CACHE.lock().await;
    let entry = guard.entry(coin).or_default();
    f(entry);
}

async fn read_stats_cache(coin: CoinId) -> Option<ExplorerStats> {
    let guard = CACHE.lock().await;
    guard
        .get(&coin)?
        .stats
        .as_ref()
        .filter(|e| e.at.elapsed() < CACHE_TTL)
        .map(|e| e.value.clone())
}

async fn write_stats_cache(coin: CoinId, stats: ExplorerStats) {
    with_coin_cache(coin, |c| {
        c.stats = Some(TimedEntry {
            at: Instant::now(),
            value: stats,
        });
    })
    .await;
}

async fn read_blocks_cache(coin: CoinId) -> Option<Vec<ExplorerBlock>> {
    let guard = CACHE.lock().await;
    guard
        .get(&coin)?
        .blocks
        .as_ref()
        .filter(|e| e.at.elapsed() < BLOCKS_CACHE_TTL)
        .map(|e| e.value.clone())
}

async fn write_blocks_cache(coin: CoinId, blocks: Vec<ExplorerBlock>) {
    with_coin_cache(coin, |c| {
        c.blocks = Some(TimedEntry {
            at: Instant::now(),
            value: blocks,
        });
    })
    .await;
}

/// Drop the cached blocks list so the next fetch hits the explorer immediately.
/// Called by the chain tip watcher when a new block arrives.
pub async fn invalidate_blocks_cache(coin: CoinId) {
    with_coin_cache(coin, |c| {
        c.blocks = None;
    })
    .await;
}

async fn read_transactions_cache(coin: CoinId) -> Option<Vec<ExplorerTransaction>> {
    let guard = CACHE.lock().await;
    guard
        .get(&coin)?
        .transactions
        .as_ref()
        .filter(|e| e.at.elapsed() < CACHE_TTL)
        .map(|e| e.value.clone())
}

async fn write_transactions_cache(coin: CoinId, txs: Vec<ExplorerTransaction>) {
    with_coin_cache(coin, |c| {
        c.transactions = Some(TimedEntry {
            at: Instant::now(),
            value: txs,
        });
    })
    .await;
}

async fn read_extraction_cache(coin: CoinId, key: &str) -> Option<Vec<ExplorerExtractionEntry>> {
    let guard = CACHE.lock().await;
    guard
        .get(&coin)?
        .extractions
        .get(key)
        .filter(|e| e.at.elapsed() < CACHE_TTL)
        .map(|e| e.value.clone())
}

async fn write_extraction_cache(
    coin: CoinId,
    key: &str,
    entries: Vec<ExplorerExtractionEntry>,
) {
    with_coin_cache(coin, |c| {
        c.extractions.insert(
            key.to_string(),
            TimedEntry {
                at: Instant::now(),
                value: entries,
            },
        );
        evict_stale_extractions(&mut c.extractions);
    })
    .await;
}

async fn read_chain_cache(coin: CoinId) -> Option<Vec<ExplorerChainTip>> {
    let guard = CACHE.lock().await;
    guard
        .get(&coin)?
        .chain_tips
        .as_ref()
        .filter(|e| e.at.elapsed() < CACHE_TTL)
        .map(|e| e.value.clone())
}

async fn write_chain_cache(coin: CoinId, tips: Vec<ExplorerChainTip>) {
    with_coin_cache(coin, |c| {
        c.chain_tips = Some(TimedEntry {
            at: Instant::now(),
            value: tips,
        });
    })
    .await;
}

async fn read_peers_cache(coin: CoinId) -> Option<Vec<ExplorerPeerEntry>> {
    let guard = CACHE.lock().await;
    guard
        .get(&coin)?
        .peers
        .as_ref()
        .filter(|e| e.at.elapsed() < PEERS_CACHE_TTL)
        .map(|e| e.value.clone())
}

async fn write_peers_cache(coin: CoinId, peers: Vec<ExplorerPeerEntry>) {
    with_coin_cache(coin, |c| {
        c.peers = Some(TimedEntry {
            at: Instant::now(),
            value: peers,
        });
    })
    .await;
}
