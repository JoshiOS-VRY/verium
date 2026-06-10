//! Full-node chain backend delegating to veriumd/vericoind wallet RPC.

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::chain::types::*;
use crate::chain::ChainBackend;
use crate::coin_profile::CoinId;
use crate::error::AppResult;
use crate::rpc::RpcClient;

const COIN_SATS: f64 = 100_000_000.0;

pub struct FullNodeRpcClient {
    coin: CoinId,
    client: RpcClient,
}

impl FullNodeRpcClient {
    pub fn new(coin: CoinId, client: RpcClient) -> Self {
        Self { coin, client }
    }
}

#[async_trait]
impl ChainBackend for FullNodeRpcClient {
    fn backend_kind(&self) -> BackendKind {
        BackendKind::FullNode
    }

    fn connection_status(&self) -> ConnectionStatus {
        ConnectionStatus::Connected
    }

    async fn get_tip(&self) -> AppResult<ChainTip> {
        let info: Value = self.client.call("getblockchaininfo", json!([])).await?;
        Ok(ChainTip {
            height: info
                .get("blocks")
                .and_then(Value::as_u64)
                .unwrap_or(0) as u32,
            hash: info
                .get("bestblockhash")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        })
    }

    async fn network_info(&self) -> AppResult<NetworkInfo> {
        let info: Value = self.client.call("getnetworkinfo", json!([])).await?;
        let relay = info
            .get("relayfee")
            .and_then(Value::as_f64)
            .unwrap_or(0.00001);
        Ok(NetworkInfo {
            relay_fee_per_kb: relay * COIN_SATS / 1000.0,
        })
    }

    async fn get_balances_per_script(&self, script_hexes: &[String]) -> AppResult<Vec<WalletBalance>> {
        let bal = self.get_balance_for_scripts(script_hexes).await?;
        Ok(vec![bal; script_hexes.len()])
    }

    async fn get_balance_for_scripts(&self, _script_hexes: &[String]) -> AppResult<WalletBalance> {
        let info: Value = self.client.call("getwalletinfo", json!([])).await?;
        let confirmed = (info.get("balance").and_then(Value::as_f64).unwrap_or(0.0) * COIN_SATS) as i64;
        let unconfirmed =
            (info.get("unconfirmed_balance").and_then(Value::as_f64).unwrap_or(0.0) * COIN_SATS) as i64;
        let immature =
            (info.get("immature_balance").and_then(Value::as_f64).unwrap_or(0.0) * COIN_SATS) as i64;
        Ok(WalletBalance {
            confirmed_sats: confirmed,
            unconfirmed_sats: unconfirmed,
            immature_sats: immature,
        })
    }

    async fn list_utxos_for_scripts(&self, _script_hexes: &[String]) -> AppResult<Vec<Utxo>> {
        let items: Vec<Value> = self
            .client
            .call("listunspent", json!([1, 9_999_999]))
            .await?;
        Ok(items
            .into_iter()
            .filter_map(|u| {
                Some(Utxo {
                    txid: u.get("txid")?.as_str()?.to_string(),
                    vout: u.get("vout")?.as_u64()? as u32,
                    value_sats: (u.get("amount")?.as_f64()? * COIN_SATS) as i64,
                    height: u.get("height").and_then(Value::as_u64).unwrap_or(0) as u32,
                    address: u
                        .get("address")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    script_hex: String::new(),
                    confirmations: u.get("confirmations")?.as_u64()? as u32,
                })
            })
            .collect())
    }

    async fn get_history_for_scripts(
        &self,
        _script_hexes: &[String],
        limit: usize,
    ) -> AppResult<Vec<WalletTx>> {
        let items: Vec<Value> = self
            .client
            .call("listtransactions", json!(["*", limit, 0]))
            .await?;
        Ok(items
            .into_iter()
            .filter_map(|t| {
                Some(WalletTx {
                    txid: t.get("txid")?.as_str()?.to_string(),
                    height: t.get("blockheight").and_then(Value::as_i64).unwrap_or(-1) as i32,
                    fee_sats: None,
                    category: t.get("category")?.as_str()?.to_string(),
                    amount: t.get("amount")?.as_f64()?,
                    address: t
                        .get("address")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    confirmations: t.get("confirmations")?.as_i64()? as i32,
                    time: t.get("time").and_then(Value::as_u64),
                    blockhash: t
                        .get("blockhash")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    blockheight: t
                        .get("blockheight")
                        .and_then(Value::as_u64)
                        .map(|h| h as u32),
                })
            })
            .collect())
    }

    async fn get_raw_tx_hex(&self, txid: &str) -> AppResult<String> {
        let hex: String = self
            .client
            .call("getrawtransaction", json!([txid, false]))
            .await?;
        Ok(hex)
    }

    async fn estimate_fee(&self, target_blocks: u32) -> AppResult<FeeRate> {
        let result: Value = self
            .client
            .call(
                "estimatesmartfee",
                json!([target_blocks.max(1)]),
            )
            .await?;
        let coins_per_kb = result
            .get("feerate")
            .and_then(Value::as_f64)
            .unwrap_or(0.0001)
            * COIN_SATS
            / 1000.0;
        Ok(FeeRate { coins_per_kb })
    }

    async fn broadcast_tx(&self, raw_hex: &str) -> AppResult<String> {
        let txid: String = self
            .client
            .call("sendrawtransaction", json!([raw_hex]))
            .await?;
        Ok(txid)
    }
}
