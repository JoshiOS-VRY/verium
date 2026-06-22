//! Chain datadir layout helpers for bootstrap import (Tauri `config.rs` subset).

use std::path::{Path, PathBuf};

use crate::coin::CoinId;
use crate::daemon_config::{DaemonConfig, verium_uses_legacy_flat};
use crate::error::{HostError, HostResult};

const MIN_BOOTSTRAP_CHAINSTATE_BYTES: u64 = 5_000_000;
const MIN_BOOTSTRAP_BLOCKS_BYTES: u64 = 50_000_000;

pub fn legacy_root_chain_dir(cfg: &DaemonConfig) -> PathBuf {
    cfg.datadir.clone()
}

pub fn unified_chain_subdir(coin: CoinId, cfg: &DaemonConfig) -> PathBuf {
    let mut p = cfg.datadir.clone();
    match cfg.chain.as_str() {
        "test" => p.push("testnet3"),
        "regtest" => p.push("regtest"),
        "binarytest-verium" | "binarytest-vericoin" => p.push(&cfg.chain),
        "main" | "verium" if coin == CoinId::Verium => p.push("verium"),
        "vericoin" | "main" if coin == CoinId::Vericoin => p.push("vericoin"),
        _ => {}
    }
    p
}

pub fn chain_data_root(coin: CoinId, cfg: &DaemonConfig) -> PathBuf {
    if verium_uses_legacy_flat(cfg) {
        cfg.datadir.clone()
    } else {
        unified_chain_subdir(coin, cfg)
    }
}

pub fn bootstrap_chain_datadir(coin: CoinId, cfg: &DaemonConfig) -> PathBuf {
    let _ = promote_root_chain_data_for_unified(coin, cfg);
    if coin == CoinId::Verium && verium_uses_legacy_flat(cfg) {
        let _ = promote_subdir_chain_data_for_legacy(coin, cfg);
        legacy_root_chain_dir(cfg)
    } else {
        chain_data_root(coin, cfg)
    }
}

fn dir_size(path: &Path) -> u64 {
    if !path.is_dir() {
        return 0;
    }
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                total += p.metadata().map(|m| m.len()).unwrap_or(0);
            } else if p.is_dir() {
                total += dir_size(&p);
            }
        }
    }
    total
}

pub fn chainstate_bytes(chainstate: &Path) -> u64 {
    dir_size(chainstate)
}

pub fn blocks_data_bytes(blocks: &Path) -> u64 {
    dir_size(blocks)
}

fn chain_dir_has_snapshot(blocks: &Path, chainstate: &Path) -> bool {
    blocks.is_dir()
        && chainstate.is_dir()
        && blocks.join("blk00000.dat").is_file()
        && chainstate.join("CURRENT").is_file()
}

fn chain_snapshot_bytes(blocks: &Path, chainstate: &Path) -> u64 {
    dir_size(blocks) + dir_size(chainstate)
}

pub fn validate_bootstrap_staging(staging: &Path) -> HostResult<()> {
    let blocks = staging.join("blocks");
    let chainstate = staging.join("chainstate");
    if !blocks.is_dir() || !chainstate.is_dir() {
        return Err(HostError::other(
            "Downloaded bootstrap zip did not contain blocks/ and chainstate/ directories.",
        ));
    }
    if !blocks.join("blk00000.dat").is_file() {
        return Err(HostError::other(
            "Bootstrap blocks/ is missing blk00000.dat — the archive may be corrupt.",
        ));
    }
    let cs = chainstate_bytes(&chainstate);
    if cs < MIN_BOOTSTRAP_CHAINSTATE_BYTES {
        return Err(HostError::other(format!(
            "Bootstrap chainstate/ is too small ({cs} bytes). \
             The archive may be corrupt, truncated, or still downloading."
        )));
    }
    let blk = blocks_data_bytes(&blocks);
    if blk < MIN_BOOTSTRAP_BLOCKS_BYTES {
        return Err(HostError::other(format!(
            "Bootstrap blocks/ is too small ({blk} bytes). \
             The archive may be corrupt, truncated, or still downloading."
        )));
    }
    Ok(())
}

pub fn chain_snapshot_needs_reindex(datadir: &Path) -> bool {
    let blocks = datadir.join("blocks");
    let chainstate = datadir.join("chainstate");
    if !blocks.join("blk00000.dat").is_file() {
        return false;
    }
    blocks_data_bytes(&blocks) > 10_000_000 && chainstate_bytes(&chainstate) < 1_000_000
}

fn binary_supports_unified_subdir(coin: CoinId, cfg: &DaemonConfig) -> bool {
    if coin == CoinId::Verium && verium_uses_legacy_flat(cfg) {
        return false;
    }
    crate::daemon_binary::resolve_daemon_binary(coin).is_some()
        || (coin == CoinId::Vericoin
            && crate::daemon_binary::resolve_daemon_binary(CoinId::Verium).is_some())
}

pub fn promote_root_chain_data_for_unified(coin: CoinId, cfg: &DaemonConfig) -> HostResult<bool> {
    if coin != CoinId::Vericoin || !binary_supports_unified_subdir(coin, cfg) {
        return Ok(false);
    }
    let root = legacy_root_chain_dir(cfg);
    let sub = unified_chain_subdir(coin, cfg);
    if root == sub {
        return Ok(false);
    }

    let root_blocks = root.join("blocks");
    let root_chainstate = root.join("chainstate");
    if !chain_dir_has_snapshot(&root_blocks, &root_chainstate) {
        return Ok(false);
    }

    let sub_blocks = sub.join("blocks");
    let sub_chainstate = sub.join("chainstate");
    let root_bytes = chain_snapshot_bytes(&root_blocks, &root_chainstate);
    let sub_bytes = chain_snapshot_bytes(&sub_blocks, &sub_chainstate);
    if sub_bytes + 50_000_000 > root_bytes && sub_bytes >= 1_000_000 {
        return Ok(false);
    }

    std::fs::create_dir_all(&sub)?;
    let mut moved = false;
    for name in [
        "blocks",
        "chainstate",
        "peers.dat",
        "debug.log",
        "banlist.dat",
        "fee_estimates.dat",
    ] {
        let src = root.join(name);
        if !src.exists() {
            continue;
        }
        let dst = sub.join(name);
        if dst.exists() {
            if dst.is_dir() {
                std::fs::remove_dir_all(&dst)?;
            } else {
                let _ = std::fs::remove_file(&dst);
            }
        }
        std::fs::rename(&src, &dst)?;
        moved = true;
    }
    Ok(moved)
}

pub fn legacy_subdir_chain_ahead(coin: CoinId, cfg: &DaemonConfig) -> bool {
    let root = legacy_root_chain_dir(cfg);
    let sub = unified_chain_subdir(coin, cfg);
    if root == sub {
        return false;
    }
    let root_blocks = root.join("blocks");
    let root_chainstate = root.join("chainstate");
    let sub_blocks = sub.join("blocks");
    let sub_chainstate = sub.join("chainstate");
    if !chain_dir_has_snapshot(&sub_blocks, &sub_chainstate) {
        return false;
    }
    let sub_bytes = chain_snapshot_bytes(&sub_blocks, &sub_chainstate);
    let root_bytes = chain_snapshot_bytes(&root_blocks, &root_chainstate);
    let sub_cs_bytes = dir_size(&sub_chainstate);
    let sub_blk_bytes = dir_size(&sub_blocks);
    if sub_blocks.join("blk00000.dat").is_file()
        && sub_blk_bytes >= MIN_BOOTSTRAP_BLOCKS_BYTES
        && root_bytes + 50_000_000 < sub_blk_bytes
    {
        return true;
    }
    if !root_blocks.join("blk00000.dat").is_file()
        && sub_blocks.join("blk00000.dat").is_file()
        && sub_blk_bytes >= MIN_BOOTSTRAP_BLOCKS_BYTES
    {
        return true;
    }
    sub_bytes + 50_000_000 > root_bytes && sub_cs_bytes >= 1_000_000
}

pub fn promote_subdir_chain_data_for_legacy(coin: CoinId, cfg: &DaemonConfig) -> HostResult<bool> {
    let root = legacy_root_chain_dir(cfg);
    let sub = unified_chain_subdir(coin, cfg);
    if root == sub || !legacy_subdir_chain_ahead(coin, cfg) {
        return Ok(false);
    }

    std::fs::create_dir_all(&root)?;
    for name in ["blocks", "chainstate"] {
        let src = sub.join(name);
        if !src.exists() {
            continue;
        }
        if name == "chainstate" && chainstate_bytes(&src) < 1_000_000 {
            continue;
        }
        let dst = root.join(name);
        if dst.exists() {
            std::fs::remove_dir_all(&dst)?;
        }
        std::fs::rename(&src, &dst)?;
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verium_legacy_bootstrap_targets_datadir_root() {
        let cfg = daemon_config::default_config(CoinId::Verium);
        assert_eq!(
            bootstrap_chain_datadir(CoinId::Verium, &cfg),
            cfg.datadir
        );
    }
}
