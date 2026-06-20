//! Peer table (`getpeerinfo`).

use crate::coin::CoinId;
use crate::context::AppContext;
use crate::error::HostResult;
use crate::model::PeerRow;
use crate::rpc::RpcClient;

pub async fn get_peer_info(ctx: &AppContext, coin: CoinId) -> HostResult<Vec<PeerRow>> {
    let endpoint = match ctx.endpoint(coin) {
        Some(ep) => ep,
        None => return Ok(vec![]),
    };
    let client = RpcClient::new(ctx.http(), &endpoint);
    let value = client.call("getpeerinfo", serde_json::json!([])).await?;
    let Some(arr) = value.as_array() else {
        return Ok(vec![]);
    };
    Ok(arr
        .iter()
        .filter_map(|p| {
            Some(PeerRow {
                addr: p.get("addr")?.as_str()?.to_string(),
                subver: p
                    .get("subver")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                inbound: p.get("inbound").and_then(|v| v.as_bool()).unwrap_or(false),
                startingheight: p
                    .get("startingheight")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
                synced_headers: p
                    .get("synced_headers")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            })
        })
        .collect())
}
