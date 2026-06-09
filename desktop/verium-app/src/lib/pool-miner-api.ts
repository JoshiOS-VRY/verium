import { invoke } from "@tauri-apps/api/core";

export interface PoolMinerStartConfig {
  stratumUrl: string;
  username: string;
  password?: string;
  threads: number;
  /** Optional comma-separated failover pool URL(s) for the sidecar backend. */
  backupUrl?: string;
}

export type PoolMinerBackend = "sidecar" | "native" | string;

export interface PoolMinerStatus {
  running: boolean;
  hashrateHm: number;
  worker: string;
  lastLogLine: string;
  /** `sidecar` — dedicated cpuminer process; `native` — hashing inside veriumd. */
  backend: PoolMinerBackend;
  activeThreads: number;
  /** Accepted shares this session (sidecar backend only). */
  acceptedShares: number;
  /** Rejected shares this session (sidecar backend only). */
  rejectedShares: number;
  /** True when the miner reports a live pool connection. */
  poolConnected: boolean;
  /** Human-readable connection state (e.g. `connected`, `restarting`). */
  connectionState: string;
}

export interface PoolMinerDetectResult {
  found: boolean;
  /** True when the running veriumd answered poolminerdetect. */
  rpcReady: boolean;
  sidecarFound: boolean;
  path?: string | null;
  source: string;
}

export interface PoolMinerMemoryLimits {
  /** Auto-adjust / cpuminer `-t 0` recommendation. */
  maxSafeThreads: number;
  /** Manual thread slider ceiling (P-logical count when hybrid). */
  maxManualThreads: number;
  scratchpadMib: number;
  totalRamMib: number;
  availableRamMib: number;
  usesSidecar: boolean;
}

export function detectPoolMiner(): Promise<PoolMinerDetectResult> {
  return invoke<PoolMinerDetectResult>("pool_miner_detect");
}

export function fetchPoolMinerStatus(): Promise<PoolMinerStatus> {
  return invoke<PoolMinerStatus>("pool_miner_status");
}

export function fetchPoolMinerMemoryLimits(): Promise<PoolMinerMemoryLimits> {
  return invoke<PoolMinerMemoryLimits>("pool_miner_memory_limits");
}

export function startPoolMiner(
  config: PoolMinerStartConfig,
): Promise<void> {
  return invoke("pool_miner_start", { config });
}

export function stopPoolMiner(): Promise<void> {
  return invoke("pool_miner_stop");
}
