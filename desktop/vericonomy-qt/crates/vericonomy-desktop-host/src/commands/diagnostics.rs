//! Daemon debug.log tail + raw RPC passthrough.

use std::path::PathBuf;

use crate::coin::CoinId;
use crate::context::AppContext;
use crate::error::{HostError, HostResult};
use crate::model::LogLine;
use crate::rpc::RpcClient;

pub async fn tail_logs(_ctx: &AppContext, coin: CoinId, max_lines: usize) -> HostResult<Vec<LogLine>> {
    let mut best: Vec<String> = Vec::new();
    for dir in log_candidate_dirs(coin) {
        let lines = tail_debug_log(&dir, max_lines).await?;
        if lines.len() > best.len() {
            best = lines;
        }
    }
    Ok(best.into_iter().map(classify_log_line).collect())
}

pub async fn rpc_raw_call(
    ctx: &AppContext,
    coin: CoinId,
    method: &str,
    params_json: &str,
) -> HostResult<String> {
    let endpoint = ctx
        .endpoint(coin)
        .ok_or_else(|| HostError::other("no RPC endpoint configured"))?;
    let client = RpcClient::new(ctx.http(), &endpoint);
    let params: serde_json::Value =
        serde_json::from_str(params_json).unwrap_or(serde_json::json!([]));
    let result = client.call(method, params).await?;
    Ok(serde_json::to_string_pretty(&result)?)
}

fn log_candidate_dirs(coin: CoinId) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(base) = crate::config::node_conf_dir() {
        dirs.push(base.join(coin.data_dir_name()));
        dirs.push(base);
    }
    dirs
}

async fn tail_debug_log(datadir: &PathBuf, max_lines: usize) -> HostResult<Vec<String>> {
    let path = datadir.join("debug.log");
    let Ok(text) = tokio::fs::read_to_string(&path).await else {
        return Ok(vec![]);
    };
    let lines: Vec<&str> = text.lines().collect();
    let take = lines.len().saturating_sub(max_lines);
    Ok(lines[take..].iter().map(|s| (*s).to_string()).collect())
}

fn classify_log_line(line: String) -> LogLine {
    let lower = line.to_ascii_lowercase();
    let level = if lower.contains("error") || lower.contains("fatal") {
        "error"
    } else if lower.contains("warn") {
        "warn"
    } else {
        "info"
    };
    LogLine {
        level: level.to_string(),
        message: line,
    }
}
