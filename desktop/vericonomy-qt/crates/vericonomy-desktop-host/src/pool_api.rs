//! Public Verium pool stats API (Tauri `pool_api.rs` subset).

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::context::AppContext;
use crate::error::{HostError, HostResult};

const POOL_STATS_URL: &str = "https://pool.vericonomy.com/api/stats";
const HTTP_USER_AGENT: &str = "vericonomy-desktop-app/1.0";

const POOL_SUPABASE_URL: &str = match option_env!("POOL_SUPABASE_URL") {
    Some(v) => v,
    None => "https://pctpdmqxideezyqxvxbj.supabase.co",
};
const POOL_SUPABASE_ANON_KEY: &str = match option_env!("POOL_SUPABASE_ANON_KEY") {
    Some(v) => v,
    None => "",
};

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

async fn supabase_get(ctx: &AppContext, path: &str, query: &str) -> HostResult<Value> {
    if POOL_SUPABASE_ANON_KEY.is_empty() {
        return Err(HostError::other("pool supabase not configured"));
    }
    let url = format!(
        "{}/rest/v1/{}{}",
        POOL_SUPABASE_URL.trim_end_matches('/'),
        path,
        query
    );
    let resp = ctx
        .http()
        .get(&url)
        .header("apikey", POOL_SUPABASE_ANON_KEY)
        .header("Authorization", format!("Bearer {}", POOL_SUPABASE_ANON_KEY))
        .header("User-Agent", HTTP_USER_AGENT)
        .send()
        .await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(HostError::other(format!(
            "pool get {path} http {status}: {text}"
        )));
    }
    Ok(resp.json().await?)
}

pub async fn fetch_pool_stats(ctx: &AppContext) -> HostResult<PoolStats> {
    let resp = ctx
        .http()
        .get(POOL_STATS_URL)
        .header("User-Agent", HTTP_USER_AGENT)
        .send()
        .await;

    if let Ok(resp) = resp {
        if resp.status().is_success() {
            let value: Value = resp.json().await?;
            return Ok(parse_pool_stats_from_public_api(&value));
        }
    }

    let value = supabase_get(ctx, "public_pool_stats", "?select=*").await?;
    let row = value
        .as_array()
        .and_then(|a| a.first())
        .cloned()
        .unwrap_or(Value::Null);
    Ok(parse_pool_stats_row(&row))
}
