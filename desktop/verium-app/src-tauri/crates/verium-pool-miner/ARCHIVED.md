# verium-pool-miner — ARCHIVED

This crate is **archived and no longer built or referenced** by the wallet. It is
not listed as a dependency of `vericonomy-wallet` (`src-tauri/Cargo.toml`) and no
module imports it.

## Why

It was a third independent Stratum/scrypt² implementation, alongside:

- `verium/src/poolminer.cpp` — the in-process miner in `veriumd` (now a degraded
  fallback only), and
- `veriumMiner` (cpuminer) — the canonical, MSVC SIMD-optimized miner that the
  public pool is tested against.

Maintaining a third client meant protocol drift and duplicate bug fixes. The
wallet now spawns the `veriumMiner` cpuminer sidecar under a Rust supervisor
(`src-tauri/src/mining_supervisor.rs`), which delivers full SIMD throughput and
isolates hashing from the node process.

## What replaced it

- Hashing engine: bundled `cpuminer-<triple>` sidecar (see
  `scripts/fetch-cpuminer.cjs`, `tauri.conf.json` `externalBin`).
- Lifecycle / telemetry: `mining_supervisor.rs` (spawn, monitor via the local
  cpuminer API, ring-buffer logs, auto-restart).
- Fallback when the sidecar binary is absent: in-process `poolminerstart` on
  `veriumd`.

## Removal

This directory can be deleted once no branch depends on its history. It is kept
only to preserve the implementation and its KAT/share-replay tests for reference.
