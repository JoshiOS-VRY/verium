import { invoke } from '@tauri-apps/api/core';

export interface PoolStats {
  poolHashrate: number | null;
  networkHashrate: number | null;
  activeMiners: number | null;
  activeWorkers: number | null;
  blocksFoundTotal: number | null;
  blocksPending: number | null;
  blocksConfirmed: number | null;
  blocksOrphaned: number | null;
  poolFeePct: number | null;
  currentDifficulty: number | null;
  currentBlockReward: number | null;
  lastBlockAt: string | null;
  capturedAt: string | null;
  source: string;
}

export interface PoolWorker {
  id?: string | null;
  name: string;
  state: string;
  current_hashrate: number | null;
  avg_hashrate: number | null;
  last_share_at: string | null;
  accepted: number;
  rejected: number;
  stale: number;
}

export interface PoolPayoutRow {
  amount_sat: string;
  txid: string | null;
  state: string;
  confirmations: number;
  created_at: string;
}

export interface MinerOverview {
  address: string;
  last_share_at: string | null;
  pending_sat: string;
  lifetime_reward_sat: string;
  lifetime_paid_sat: string;
  workers: PoolWorker[];
  recent_payouts: PoolPayoutRow[];
}

export interface HashratePoint {
  ts: string;
  hashrate: number;
}

export interface PoolPayoutSummary {
  pending_sat: string;
  lifetime_reward_sat: string;
  lifetime_paid_sat: string;
  last_auto_cycle_at: string | null;
  last_auto_batch_at: string | null;
  payouts_paused: boolean;
}

export function isPoolApiEnabled(): Promise<boolean> {
  return invoke<boolean>('is_pool_api_enabled_cmd');
}

export function fetchPoolStats(): Promise<PoolStats> {
  return invoke<PoolStats>('fetch_pool_stats_cmd');
}

export function fetchMinerOverview(address: string): Promise<MinerOverview | null> {
  return invoke<MinerOverview | null>('fetch_miner_overview_cmd', { address });
}

export function fetchMinerHashrateHistory(
  address: string,
  hours = 12,
  bucketSeconds = 30,
  smoothSeconds = 1800
): Promise<HashratePoint[]> {
  return invoke<HashratePoint[]>('fetch_miner_hashrate_history_cmd', {
    address,
    hours,
    bucketSeconds,
    smoothSeconds,
  });
}

export function fetchPoolPayoutSummary(): Promise<PoolPayoutSummary> {
  return invoke<PoolPayoutSummary>('fetch_pool_payout_summary_cmd');
}

export interface MinerPayoutsResult {
  rows: PoolPayoutRow[];
  total: number;
}

export function fetchMinerPayouts(
  address: string,
  limit = 20,
  offset = 0
): Promise<MinerPayoutsResult> {
  return invoke<MinerPayoutsResult>('fetch_miner_payouts_cmd', {
    address,
    limit,
    offset,
  });
}
