import { invoke } from "@tauri-apps/api/core";

export interface PoolMinerStartConfig {
  stratumUrl: string;
  username: string;
  password?: string;
  threads: number;
}

export interface PoolMinerStatus {
  running: boolean;
  hashrateHm: number;
  worker: string;
  lastLogLine: string;
  /** `native` — hashing inside veriumd. */
  backend: string;
  activeThreads: number;
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
  maxSafeThreads: number;
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

export function fetchPoolMinerLogLines(maxLines = 120): Promise<string[]> {
  return invoke<string[]>("pool_miner_log_lines", { maxLines });
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
