//! Pluggable chain backends: local full node RPC or Electrum light client.

pub mod electrum;
pub mod full_node;
pub mod tx_hex;
pub mod types;

use async_trait::async_trait;

pub use vericonomy_chain::ChainBackend as SdkChainBackend;
pub use types::*;

use crate::coin_profile::CoinId;
use crate::error::AppResult;
use crate::sdk_bridge::map_wallet_err;

/// Remote read + broadcast surface shared by full-node and light wallets.
#[async_trait]
pub trait ChainBackend: Send + Sync {
    fn backend_kind(&self) -> BackendKind;
    fn connection_status(&self) -> ConnectionStatus;

    async fn get_tip(&self) -> AppResult<ChainTip>;
    async fn network_info(&self) -> AppResult<NetworkInfo>;

    async fn get_balance_for_scripts(&self, script_hexes: &[String]) -> AppResult<WalletBalance>;
    /// Per-script balances in the same order as `script_hexes` (for parallel gap scan).
    async fn get_balances_per_script(
        &self,
        script_hexes: &[String],
    ) -> AppResult<Vec<WalletBalance>>;
    async fn list_utxos_for_scripts(&self, script_hexes: &[String]) -> AppResult<Vec<Utxo>>;
    async fn get_history_for_scripts(
        &self,
        script_hexes: &[String],
        limit: usize,
    ) -> AppResult<Vec<WalletTx>>;

    /// Fill in amount/time/address on history rows (Electrum only). No-op on full node.
    async fn enrich_tx_history_batch(
        &self,
        _coin: CoinId,
        _script_hexes: &[String],
        _txs: &mut [WalletTx],
        _max_rows: usize,
    ) -> AppResult<()> {
        Ok(())
    }

    async fn get_raw_tx_hex(&self, txid: &str) -> AppResult<String>;
    async fn estimate_fee(&self, target_blocks: u32) -> AppResult<FeeRate>;
    async fn broadcast_tx(&self, raw_hex: &str) -> AppResult<String>;

    /// Light-mode only: current Electrum server status.
    async fn light_server_status(&self) -> Option<LightServerStatus> {
        None
    }

    /// Cap Electrum RPC rate during initial HD indexing. No-op on full node.
    async fn set_initial_indexing_limits(
        &self,
        _max_scripthash_rpcs: u32,
        _max_scripts_per_batch: u32,
    ) {
    }

    /// True when the indexing RPC budget for this sync slice is exhausted.
    fn indexing_budget_exhausted(&self) -> bool {
        false
    }

    /// Clear indexing limits after a sync slice completes.
    async fn clear_initial_indexing_limits(&self) {}
}

/// Adapts an SDK [`SdkChainBackend`] to the app-facing [`ChainBackend`] (`AppResult`).
pub struct SdkChainAdapter<B>(pub B);

#[async_trait]
impl<B: SdkChainBackend + Send + Sync> ChainBackend for SdkChainAdapter<B> {
    fn backend_kind(&self) -> BackendKind {
        self.0.backend_kind()
    }

    fn connection_status(&self) -> ConnectionStatus {
        self.0.connection_status()
    }

    async fn get_tip(&self) -> AppResult<ChainTip> {
        map_wallet_err(self.0.get_tip().await)
    }

    async fn network_info(&self) -> AppResult<NetworkInfo> {
        map_wallet_err(self.0.network_info().await)
    }

    async fn get_balance_for_scripts(&self, script_hexes: &[String]) -> AppResult<WalletBalance> {
        map_wallet_err(self.0.get_balance_for_scripts(script_hexes).await)
    }

    async fn get_balances_per_script(
        &self,
        script_hexes: &[String],
    ) -> AppResult<Vec<WalletBalance>> {
        map_wallet_err(self.0.get_balances_per_script(script_hexes).await)
    }

    async fn list_utxos_for_scripts(&self, script_hexes: &[String]) -> AppResult<Vec<Utxo>> {
        map_wallet_err(self.0.list_utxos_for_scripts(script_hexes).await)
    }

    async fn get_history_for_scripts(
        &self,
        script_hexes: &[String],
        limit: usize,
    ) -> AppResult<Vec<WalletTx>> {
        map_wallet_err(self.0.get_history_for_scripts(script_hexes, limit).await)
    }

    async fn enrich_tx_history_batch(
        &self,
        coin: CoinId,
        script_hexes: &[String],
        txs: &mut [WalletTx],
        max_rows: usize,
    ) -> AppResult<()> {
        map_wallet_err(
            self.0
                .enrich_tx_history_batch(coin, script_hexes, txs, max_rows)
                .await,
        )
    }

    async fn get_raw_tx_hex(&self, txid: &str) -> AppResult<String> {
        map_wallet_err(self.0.get_raw_tx_hex(txid).await)
    }

    async fn estimate_fee(&self, target_blocks: u32) -> AppResult<FeeRate> {
        map_wallet_err(self.0.estimate_fee(target_blocks).await)
    }

    async fn broadcast_tx(&self, raw_hex: &str) -> AppResult<String> {
        map_wallet_err(self.0.broadcast_tx(raw_hex).await)
    }

    async fn light_server_status(&self) -> Option<LightServerStatus> {
        self.0.light_server_status().await
    }

    async fn set_initial_indexing_limits(
        &self,
        max_scripthash_rpcs: u32,
        max_scripts_per_batch: u32,
    ) {
        self.0
            .set_initial_indexing_limits(max_scripthash_rpcs, max_scripts_per_batch)
            .await
    }

    fn indexing_budget_exhausted(&self) -> bool {
        self.0.indexing_budget_exhausted()
    }

    async fn clear_initial_indexing_limits(&self) {
        self.0.clear_initial_indexing_limits().await
    }
}

pub fn script_hex_from_address(coin: CoinId, address: &str) -> AppResult<String> {
    map_wallet_err(vericonomy_chain::script_hex_from_address(coin, address))
}
