//! Callback during Electrum gap scan to persist balance incrementally.

use async_trait::async_trait;

use crate::error::AppResult;

#[async_trait]
pub trait GapScanHook: Send + Sync {
    /// `utxo_refresh` false during indexing slices to conserve Electrum RPC budget.
    async fn on_funded_batch(&self, funded: &[String], utxo_refresh: bool) -> AppResult<()>;
}
