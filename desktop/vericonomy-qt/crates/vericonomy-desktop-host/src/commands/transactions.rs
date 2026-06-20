//! Transaction list + cumulative balance series.

use serde_json::Value;

use crate::coin::CoinId;
use crate::context::AppContext;
use crate::error::HostResult;
use crate::model::{BalancePoint, TransactionRow};
use crate::rpc::RpcClient;

/// Max wallet rows fetched for the history table (newest entries when capped).
pub const TRANSACTIONS_LIST_CAP: u32 = 500;

pub struct TransactionList {
    pub rows: Vec<TransactionRow>,
    pub balance_series: Vec<BalancePoint>,
}

impl Default for TransactionList {
    fn default() -> Self {
        Self {
            rows: vec![],
            balance_series: vec![],
        }
    }
}

/// `listtransactions` count/skip to load the newest up-to-cap wallet rows.
pub fn list_transactions_fetch_params(total_count: i64) -> (u32, u32) {
    if total_count <= 0 {
        return (0, 0);
    }
    let total = total_count as u32;
    let count = total.min(TRANSACTIONS_LIST_CAP);
    let skip = total.saturating_sub(count);
    (count, skip)
}

pub async fn list_transactions(
    ctx: &AppContext,
    coin: CoinId,
    count: u32,
    skip: u32,
) -> HostResult<TransactionList> {
    let endpoint = match ctx.endpoint(coin) {
        Some(ep) => ep,
        None => return Ok(TransactionList::default()),
    };
    let client = RpcClient::new(ctx.http(), &endpoint);
    let value = client
        .call("listtransactions", serde_json::json!(["*", count, skip]))
        .await?;

    let mut rows = parse_rows(&value);
    rows.sort_by_key(|r| r.time);
    let balance_series = build_balance_series(&rows);
    rows.sort_by(|a, b| b.time.cmp(&a.time).then_with(|| b.txid.cmp(&a.txid)));
    Ok(TransactionList { rows, balance_series })
}

fn parse_rows(value: &Value) -> Vec<TransactionRow> {
    let Some(arr) = value.as_array() else {
        return vec![];
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    arr.iter()
        .filter_map(|tx| {
            let amount = tx.get("amount")?.as_f64()?;
            let time = tx.get("time")?.as_i64().unwrap_or(0);
            Some(TransactionRow {
                txid: tx
                    .get("txid")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                category: tx
                    .get("category")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                amount,
                fee: tx.get("fee").and_then(|v| v.as_f64()),
                confirmations: tx.get("confirmations").and_then(|v| v.as_i64()).unwrap_or(0),
                address: tx
                    .get("address")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
                time,
                time_label: format_time_ago(time, now),
                time_display: format_transaction_time(time),
            })
        })
        .collect()
}

fn format_transaction_time(tx_time: i64) -> String {
    if tx_time <= 0 {
        return "—".to_string();
    }
    // RFC3339-style local display without adding chrono dependency.
    let secs = tx_time;
    let days = secs / 86400;
    let rem = secs % 86400;
    let hours = rem / 3600;
    let minutes = (rem % 3600) / 60;
    // Approximate calendar date from Unix epoch (good enough for wallet UI).
    let (y, m, d) = epoch_to_ymd(days);
    format!("{y:04}-{m:02}-{d:02} {hours:02}:{minutes:02}")
}

fn epoch_to_ymd(mut days: i64) -> (i64, i64, i64) {
    let mut y = 1970i64;
    loop {
        let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
        let year_days = if leap { 366 } else { 365 };
        if days < year_days {
            break;
        }
        days -= year_days;
        y += 1;
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 1i64;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        m += 1;
    }
    (y, m, days + 1)
}

fn format_time_ago(tx_time: i64, now: i64) -> String {
    if tx_time <= 0 {
        return "—".to_string();
    }
    let delta = (now - tx_time).max(0);
    if delta < 60 {
        return format!("{delta}s ago");
    }
    if delta < 3600 {
        return format!("{}m ago", delta / 60);
    }
    if delta < 86400 {
        return format!("{}h ago", delta / 3600);
    }
    format!("{}d ago", delta / 86400)
}

/// Cumulative balance over time (oldest tx first), matching the React chart logic.
fn build_balance_series(rows: &[TransactionRow]) -> Vec<BalancePoint> {
    if rows.is_empty() {
        return vec![];
    }
    let mut cumulative = 0.0;
    let mut points = Vec::with_capacity(rows.len());
    for row in rows {
        cumulative += row.amount;
        points.push(BalancePoint {
            t: row.time * 1000,
            balance: cumulative.max(0.0),
        });
    }
    points
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_params_caps_at_500() {
        assert_eq!(list_transactions_fetch_params(0), (0, 0));
        assert_eq!(list_transactions_fetch_params(10), (10, 0));
        assert_eq!(list_transactions_fetch_params(600), (500, 100));
    }
}
