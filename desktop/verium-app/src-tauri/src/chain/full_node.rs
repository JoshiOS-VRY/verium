//! Full-node chain backend delegating to veriumd/vericoind wallet RPC via the SDK.

use async_trait::async_trait;
use serde_json::Value;
use vericonomy_chain::full_node::{FullNodeRpcClient as SdkFullNodeClient, JsonRpcClient};

use crate::chain::types::*;
use crate::chain::{ChainBackend, SdkChainAdapter};
use crate::coin_profile::CoinId;
use crate::error::AppResult;
use crate::rpc::RpcClient;

/// Bridges the app's [`RpcClient`] to the SDK [`JsonRpcClient`] trait.
pub struct RpcJsonAdapter(pub RpcClient);

#[async_trait]
impl JsonRpcClient for RpcJsonAdapter {
    async fn call(&self, method: &str, params: Value) -> vericonomy_errors::Result<Value> {
        self.0
            .call(method, params)
            .await
            .map_err(|e| e.into())
    }
}

pub struct FullNodeRpcClient {
    inner: SdkChainAdapter<SdkFullNodeClient<RpcJsonAdapter>>,
}

impl FullNodeRpcClient {
    pub fn new(coin: CoinId, client: RpcClient) -> Self {
        Self {
            inner: SdkChainAdapter(SdkFullNodeClient::new(
                coin,
                RpcJsonAdapter(client),
            )),
        }
    }
}

#[async_trait]
impl ChainBackend for FullNodeRpcClient {
    fn backend_kind(&self) -> BackendKind {
        self.inner.backend_kind()
    }

    fn connection_status(&self) -> ConnectionStatus {
        self.inner.connection_status()
    }

    async fn get_tip(&self) -> AppResult<ChainTip> {
        self.inner.get_tip().await
    }

    async fn network_info(&self) -> AppResult<NetworkInfo> {
        self.inner.network_info().await
    }

    async fn get_balance_for_scripts(&self, script_hexes: &[String]) -> AppResult<WalletBalance> {
        self.inner.get_balance_for_scripts(script_hexes).await
    }

    async fn get_balances_per_script(
        &self,
        script_hexes: &[String],
    ) -> AppResult<Vec<WalletBalance>> {
        self.inner.get_balances_per_script(script_hexes).await
    }

    async fn list_utxos_for_scripts(&self, script_hexes: &[String]) -> AppResult<Vec<Utxo>> {
        self.inner.list_utxos_for_scripts(script_hexes).await
    }

    async fn get_history_for_scripts(
        &self,
        script_hexes: &[String],
        limit: usize,
    ) -> AppResult<Vec<WalletTx>> {
        self.inner.get_history_for_scripts(script_hexes, limit).await
    }

    async fn get_raw_tx_hex(&self, txid: &str) -> AppResult<String> {
        self.inner.get_raw_tx_hex(txid).await
    }

    async fn estimate_fee(&self, target_blocks: u32) -> AppResult<FeeRate> {
        self.inner.estimate_fee(target_blocks).await
    }

    async fn broadcast_tx(&self, raw_hex: &str) -> AppResult<String> {
        self.inner.broadcast_tx(raw_hex).await
    }
}
