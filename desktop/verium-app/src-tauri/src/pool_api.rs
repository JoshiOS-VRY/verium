use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::Mutex;

use crate::error::{AppError, AppResult};
use crate::http_shared::shared_http_client;

const POOL_STATS_URL: &str = "https://pool.vericonomy.com/api/stats";
const CACHE_TTL: Duration = Duration::from_secs(30);
const MAX_MINER_CACHE_KEYS: usize = 16;
const MAX_HASHRATE_CACHE_KEYS: usize = 16;
const HTTP_TIMEOUT: Duration = Duration::from_secs(12);
const HTTP_USER_AGENT: &str = "vericonomy-desktop-app/1.0";

static POOL_SUPABASE_URL: &str = env!("POOL_SUPABASE_URL");
static POOL_SUPABASE_ANON_KEY: &str = env!("POOL_SUPABASE_ANON_KEY");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolStats {
    pub pool_hashrate: Option<f64>,
    pub network_hashrate: Option<f64>,
    pub active_miners: Option<u64>,
    pub active_workers: Option<u64>,
    pub blocks_found_total: Option<u64>,
    pub blocks_pending: Option<u64>,
    pub blocks_confirmed: Option<u64>,
    pub blocks_orphaned: Option<u64>,
    pub pool_fee_pct: Option<f64>,
    pub current_difficulty: Option<f64>,
    pub current_block_reward: Option<f64>,
    pub last_block_at: Option<String>,
    pub captured_at: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PoolWorker {
    pub id: Option<String>,
    pub name: String,
    pub state: String,
    pub current_hashrate: Option<f64>,
    pub avg_hashrate: Option<f64>,
    pub last_share_at: Option<String>,
    pub accepted: u64,
    pub rejected: u64,
    pub stale: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PoolPayoutRow {
    pub amount_sat: String,
    pub txid: Option<String>,
    pub state: String,
    pub confirmations: u64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MinerOverview {
    pub address: String,
    pub last_share_at: Option<String>,
    pub pending_sat: String,
    pub lifetime_reward_sat: String,
    pub lifetime_paid_sat: String,
    pub workers: Vec<PoolWorker>,
    pub recent_payouts: Vec<PoolPayoutRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashratePoint {
    pub ts: String,
    pub hashrate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PoolPayoutSummary {
    pub pending_sat: String,
    pub lifetime_reward_sat: String,
    pub lifetime_paid_sat: String,
    pub last_auto_cycle_at: Option<String>,
    pub last_auto_batch_at: Option<String>,
    pub payouts_paused: bool,
}

struct TimedEntry<T> {
    at: Instant,
    value: T,
}

struct PoolCache {
    stats: Option<TimedEntry<PoolStats>>,
    miners: HashMap<String, TimedEntry<MinerOverview>>,
    hashrate: HashMap<String, TimedEntry<Vec<HashratePoint>>>,
    payout_summary: Option<TimedEntry<PoolPayoutSummary>>,
}

impl Default for PoolCache {
    fn default() -> Self {
        Self {
            stats: None,
            miners: HashMap::new(),
            hashrate: HashMap::new(),
            payout_summary: None,
        }
    }
}

static CACHE: once_cell::sync::Lazy<Mutex<PoolCache>> =
    once_cell::sync::Lazy::new(|| Mutex::new(PoolCache::default()));

pub fn is_pool_api_enabled() -> bool {
    !POOL_SUPABASE_ANON_KEY.is_empty()
}

fn http_client() -> AppResult<reqwest::Client> {
    shared_http_client(HTTP_TIMEOUT, HTTP_USER_AGENT)
}

fn evict_timed_map<K, V>(map: &mut HashMap<K, TimedEntry<V>>, max_keys: usize)
where
    K: Clone + Eq + std::hash::Hash,
{
    map.retain(|_, entry| entry.at.elapsed() < CACHE_TTL);
    while map.len() > max_keys {
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

fn supabase_configured() -> AppResult<()> {
    if POOL_SUPABASE_ANON_KEY.is_empty() {
        return Err(AppError::other(
            "pool supabase not configured: set POOL_SUPABASE_ANON_KEY at build time",
        ));
    }
    Ok(())
}

async fn supabase_rpc(client: &reqwest::Client, fn_name: &str, body: Value) -> AppResult<Value> {
    supabase_configured()?;
    let url = format!(
        "{}/rest/v1/rpc/{}",
        POOL_SUPABASE_URL.trim_end_matches('/'),
        fn_name
    );
    let resp = client
        .post(&url)
        .header("apikey", POOL_SUPABASE_ANON_KEY)
        .header("Authorization", format!("Bearer {}", POOL_SUPABASE_ANON_KEY))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::other(format!(
            "pool rpc {fn_name} http {status}: {text}"
        )));
    }
    Ok(resp.json().await?)
}

async fn supabase_get(
    client: &reqwest::Client,
    path: &str,
    query: &str,
) -> AppResult<Value> {
    supabase_configured()?;
    let url = format!(
        "{}/rest/v1/{}{}",
        POOL_SUPABASE_URL.trim_end_matches('/'),
        path,
        query
    );
    let resp = client
        .get(&url)
        .header("apikey", POOL_SUPABASE_ANON_KEY)
        .header("Authorization", format!("Bearer {}", POOL_SUPABASE_ANON_KEY))
        .send()
        .await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::other(format!(
            "pool get {path} http {status}: {text}"
        )));
    }
    Ok(resp.json().await?)
}

fn parse_pool_stats_from_public_api(value: &Value) -> PoolStats {
    let f = |k: &str| value.get(k).and_then(|v| v.as_f64());
    let u = |k: &str| value.get(k).and_then(|v| v.as_u64());
    let s = |k: &str| {
        value
            .get(k)
            .and_then(|v| v.as_str())
            .map(|x| x.to_string())
    };
    PoolStats {
        pool_hashrate: f("hashrate"),
        network_hashrate: f("network_hashrate"),
        active_miners: u("miners").or_else(|| u("minercount")),
        active_workers: u("workers").or_else(|| u("workercount")),
        blocks_found_total: u("blocks_found_total"),
        blocks_pending: u("blocks_pending"),
        blocks_confirmed: u("blocks_confirmed"),
        blocks_orphaned: u("blocks_orphaned"),
        pool_fee_pct: f("fee_percent"),
        current_difficulty: f("current_difficulty"),
        current_block_reward: f("current_block_reward"),
        last_block_at: s("last_block_at"),
        captured_at: s("updated_at"),
        source: "pool-api".to_string(),
    }
}

fn parse_pool_stats_row(value: &Value) -> PoolStats {
    PoolStats {
        pool_hashrate: value.get("pool_hashrate").and_then(|v| v.as_f64()),
        network_hashrate: value.get("network_hashrate").and_then(|v| v.as_f64()),
        active_miners: value.get("active_miners").and_then(|v| v.as_u64()),
        active_workers: value.get("active_workers").and_then(|v| v.as_u64()),
        blocks_found_total: value.get("blocks_found_total").and_then(|v| v.as_u64()),
        blocks_pending: value.get("blocks_pending").and_then(|v| v.as_u64()),
        blocks_confirmed: value.get("blocks_confirmed").and_then(|v| v.as_u64()),
        blocks_orphaned: value.get("blocks_orphaned").and_then(|v| v.as_u64()),
        pool_fee_pct: value.get("pool_fee_pct").and_then(|v| v.as_f64()),
        current_difficulty: value.get("current_difficulty").and_then(|v| v.as_f64()),
        current_block_reward: value
            .get("current_block_reward")
            .and_then(|v| v.as_f64()),
        last_block_at: value
            .get("last_block_at")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        captured_at: value
            .get("captured_at")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        source: "supabase".to_string(),
    }
}

pub async fn fetch_pool_stats() -> AppResult<PoolStats> {
    {
        let cache = CACHE.lock().await;
        if let Some(entry) = &cache.stats {
            if entry.at.elapsed() < CACHE_TTL {
                return Ok(entry.value.clone());
            }
        }
    }

    let client = http_client()?;
    let stats = match client.get(POOL_STATS_URL).send().await {
        Ok(resp) if resp.status().is_success() => {
            let value: Value = resp.json().await?;
            parse_pool_stats_from_public_api(&value)
        }
        _ => {
            if !is_pool_api_enabled() {
                return Err(AppError::other("pool stats unavailable"));
            }
            let value = supabase_get(&client, "public_pool_stats", "?select=*").await?;
            let row = value
                .as_array()
                .and_then(|a| a.first())
                .cloned()
                .unwrap_or(Value::Null);
            parse_pool_stats_row(&row)
        }
    };

    let mut cache = CACHE.lock().await;
    cache.stats = Some(TimedEntry {
        at: Instant::now(),
        value: stats.clone(),
    });
    Ok(stats)
}

pub async fn fetch_miner_overview(address: String) -> AppResult<Option<MinerOverview>> {
    let key = address.trim().to_string();
    if key.is_empty() {
        return Ok(None);
    }

    {
        let cache = CACHE.lock().await;
        if let Some(entry) = cache.miners.get(&key) {
            if entry.at.elapsed() < CACHE_TTL {
                return Ok(Some(entry.value.clone()));
            }
        }
    }

    let client = http_client()?;
    let value = supabase_rpc(
        &client,
        "miner_overview",
        json!({ "p_address": key }),
    )
    .await?;

    if value.is_null() {
        return Ok(None);
    }

    let overview: MinerOverview = serde_json::from_value(value)?;
    let mut cache = CACHE.lock().await;
    cache.miners.insert(
        key,
        TimedEntry {
            at: Instant::now(),
            value: overview.clone(),
        },
    );
    evict_timed_map(&mut cache.miners, MAX_MINER_CACHE_KEYS);
    Ok(Some(overview))
}

pub async fn fetch_miner_hashrate_history(
    address: String,
    hours: Option<u32>,
    bucket_seconds: Option<u32>,
    smooth_seconds: Option<u32>,
) -> AppResult<Vec<HashratePoint>> {
    let key = format!(
        "{}:{}:{}:{}",
        address.trim(),
        hours.unwrap_or(12),
        bucket_seconds.unwrap_or(30),
        smooth_seconds.unwrap_or(1800)
    );

    {
        let cache = CACHE.lock().await;
        if let Some(entry) = cache.hashrate.get(&key) {
            if entry.at.elapsed() < CACHE_TTL {
                return Ok(entry.value.clone());
            }
        }
    }

    let client = http_client()?;
    let value = supabase_rpc(
        &client,
        "miner_hashrate_history",
        json!({
            "p_address": address.trim(),
            "p_hours": hours.unwrap_or(12),
            "p_bucket_seconds": bucket_seconds.unwrap_or(30),
            "p_smooth_seconds": smooth_seconds.unwrap_or(1800),
        }),
    )
    .await?;
    let points = normalize_hashrate_points(&value);

    let mut cache = CACHE.lock().await;
    cache.hashrate.insert(
        key,
        TimedEntry {
            at: Instant::now(),
            value: points.clone(),
        },
    );
    evict_timed_map(&mut cache.hashrate, MAX_HASHRATE_CACHE_KEYS);
    Ok(points)
}

pub async fn fetch_pool_payout_summary() -> AppResult<PoolPayoutSummary> {
    {
        let cache = CACHE.lock().await;
        if let Some(entry) = &cache.payout_summary {
            if entry.at.elapsed() < CACHE_TTL {
                return Ok(entry.value.clone());
            }
        }
    }

    let client = http_client()?;
    let value = supabase_rpc(&client, "pool_payout_summary", json!({})).await?;
    let summary: PoolPayoutSummary = serde_json::from_value(value)?;

    let mut cache = CACHE.lock().await;
    cache.payout_summary = Some(TimedEntry {
        at: Instant::now(),
        value: summary.clone(),
    });
    Ok(summary)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinerPayoutsResult {
    pub rows: Vec<PoolPayoutRow>,
    pub total: u64,
}

pub async fn fetch_miner_payouts(
    address: String,
    limit: u32,
    offset: u32,
) -> AppResult<MinerPayoutsResult> {
    let client = http_client()?;
    let addr = urlencoding::encode(address.trim());
    let query = format!(
        "?address=eq.{addr}&select=amount_sat,txid,state,confirmations,created_at&order=created_at.desc&limit={limit}&offset={offset}",
        addr = addr,
        limit = limit,
        offset = offset
    );
    let value = supabase_get(&client, "public_payouts", &query).await?;
    let rows: Vec<PoolPayoutRow> = serde_json::from_value(value)?;

    let count_query = format!("?address=eq.{addr}&select=amount_sat", addr = addr);
    let count_resp = client
        .get(format!(
            "{}/rest/v1/public_payouts{}",
            POOL_SUPABASE_URL.trim_end_matches('/'),
            count_query
        ))
        .header("apikey", POOL_SUPABASE_ANON_KEY)
        .header("Authorization", format!("Bearer {}", POOL_SUPABASE_ANON_KEY))
        .header("Prefer", "count=exact")
        .send()
        .await?;

    let total = count_resp
        .headers()
        .get("content-range")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split('/').nth(1))
        .and_then(|n| n.parse::<u64>().ok())
        .unwrap_or(rows.len() as u64);

    Ok(MinerPayoutsResult { rows, total })
}

fn normalize_hashrate_points(data: &Value) -> Vec<HashratePoint> {
    let arr = match data {
        Value::Array(a) => a.clone(),
        Value::String(s) => serde_json::from_str(s).unwrap_or_default(),
        _ => return vec![],
    };
    arr.into_iter()
        .filter_map(|row| {
            let ts = row.get("ts")?.as_str()?.to_string();
            let hashrate = row
                .get("hashrate")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            Some(HashratePoint { ts, hashrate })
        })
        .collect()
}
