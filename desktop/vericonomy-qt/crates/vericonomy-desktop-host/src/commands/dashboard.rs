//! Dashboard snapshot — aggregates data for DashboardHero parity.

use crate::coin::CoinId;
use crate::context::AppContext;
use crate::error::HostResult;
use crate::model::DashboardSnapshot;
use crate::prefs;

fn network_hash_to_khm(network_hash_ps: f64) -> f64 {
    (network_hash_ps * 60.0) / 1000.0
}

fn resolve_block_time_min(
    explorer_blocks_per_hour: Option<f64>,
    explorer_block_time_min: Option<f64>,
    mining_blocks_per_hour: f64,
    mining_block_time: f64,
) -> f64 {
    if let Some(bph) = explorer_blocks_per_hour.filter(|v| *v > 0.0) {
        return 60.0 / bph;
    }
    if mining_blocks_per_hour > 0.0 {
        return 60.0 / mining_blocks_per_hour;
    }
    if let Some(bt) = explorer_block_time_min.filter(|v| *v > 0.0) {
        return bt;
    }
    if mining_block_time > 0.0 {
        return mining_block_time;
    }
    0.0
}

pub async fn get_dashboard_snapshot(ctx: &AppContext, coin: CoinId) -> HostResult<DashboardSnapshot> {
    let prefs = prefs::load_prefs()?;
    let is_light = prefs::wallet_mode_for(&prefs, coin).is_light();

    let node = crate::commands::node::get_node_status(ctx, coin).await?;
    let wallet = crate::commands::wallet::get_wallet_info(ctx, coin).await?;
    let explorer = crate::commands::explorer::fetch_explorer_stats(ctx, coin)
        .await
        .unwrap_or_default();

    let mining = crate::commands::mining::get_mining_info(ctx, coin).await.unwrap_or_default();
    let earn = crate::commands::mining::get_miner_state(ctx, coin);
    let staking = crate::commands::staking::get_staking_info(ctx, coin)
        .await
        .unwrap_or_default();

    let txs = crate::commands::transactions::list_transactions(ctx, coin, 500, 0)
        .await
        .unwrap_or_default();
    let blocks_found = txs
        .rows
        .iter()
        .filter(|t| t.category == "generate" || t.category == "immature")
        .count() as i64;
    let stake_rewards = txs
        .rows
        .iter()
        .filter(|t| {
            matches!(
                t.category.as_str(),
                "stake" | "stake-mint" | "stake-orphan"
            )
        })
        .count() as i64;

    let sync_target = node.headers.max(node.blocks);
    let behind = (sync_target - node.blocks).max(0);
    let synced = node.connected
        && !node.initial_block_download
        && node.verification_progress >= 0.9999
        && behind == 0;

    let state_label = if is_light {
        if node.connected {
            "Light wallet online".into()
        } else {
            "Light wallet offline".into()
        }
    } else if synced {
        "Fully synced".into()
    } else if node.initial_block_download {
        "Syncing".into()
    } else if node.connected {
        "Connected".into()
    } else {
        node.state.clone()
    };

    let activity_title = if synced || is_light {
        String::new()
    } else if node.initial_block_download {
        "Syncing blockchain…".into()
    } else {
        "Connecting to network…".into()
    };

    let network_hash_hs = explorer
        .network_hash
        .unwrap_or(if mining.networkhashps > 0.0 {
            mining.networkhashps
        } else {
            0.0
        });
    let local_hashrate_hm = if earn.active { mining.hashrate } else { 0.0 };
    let network_share_percent = if local_hashrate_hm > 0.0 && network_hash_hs > 0.0 {
        (local_hashrate_hm / (network_hash_hs * 60.0)) * 100.0
    } else {
        0.0
    };

    let blocks_per_hour = explorer
        .blocks_per_hour
        .unwrap_or(if mining.blocksperhour > 0.0 {
            mining.blocksperhour
        } else {
            0.0
        });
    let block_reward = explorer
        .block_reward
        .unwrap_or(if mining.blockreward > 0.0 {
            mining.blockreward
        } else {
            0.0
        });
    let est_daily_vrm = if local_hashrate_hm > 0.0
        && network_hash_hs > 0.0
        && blocks_per_hour > 0.0
        && block_reward > 0.0
    {
        let share = local_hashrate_hm / (network_hash_hs * 60.0);
        share * blocks_per_hour * 24.0 * block_reward
    } else {
        0.0
    };

    let block_time_min = resolve_block_time_min(
        explorer.blocks_per_hour,
        explorer.block_time_min,
        mining.blocksperhour,
        mining.blocktime,
    );

    let network_hash_khm = if network_hash_hs > 0.0 {
        network_hash_to_khm(network_hash_hs)
    } else {
        0.0
    };

    let mempool = explorer
        .pooled_tx
        .unwrap_or(if mining.pooledtx > 0 { mining.pooledtx } else { 0 });

    let peer_status = if node.connections > 0 {
        "Online".into()
    } else if node.connected {
        "No peers".into()
    } else {
        "Offline".into()
    };

    Ok(DashboardSnapshot {
        is_light,
        connected: if is_light {
            wallet_exists_light(coin).await.unwrap_or(false)
        } else {
            node.connected
        },
        synced,
        state_label,
        network_mode: if node.chain == "test" {
            "testnet".into()
        } else {
            "mainnet".into()
        },
        activity_title,
        local_blocks: node.blocks,
        sync_target,
        behind,
        verification_progress: node.verification_progress,
        connections: node.connections,
        wallet,
        explorer,
        mining_active: earn.active,
        miner_active: earn.active,
        local_hashrate: local_hashrate_hm,
        blocks_found: if coin == CoinId::Verium {
            blocks_found
        } else {
            stake_rewards
        },
        staking_active: staking.enabled,
        stake_weight: staking.stake_weight,
        net_stake_weight: staking.netstakeweight,
        network_share_percent,
        est_daily_vrm,
        block_time_min,
        network_hash_khm,
        peer_status,
        mempool,
    })
}

async fn wallet_exists_light(coin: CoinId) -> HostResult<bool> {
    crate::light_session::light_wallet_exists(coin).await
}
