//! Public pool dashboard stats.

use crate::context::AppContext;
use crate::error::HostResult;
use crate::pool_api::{fetch_pool_stats as fetch, PoolStats};

pub async fn fetch_pool_stats(ctx: &AppContext) -> HostResult<PoolStats> {
    fetch(ctx).await
}
