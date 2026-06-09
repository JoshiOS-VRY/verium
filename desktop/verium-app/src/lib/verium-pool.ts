/** Public Verium pool constants (aligned with explorer + pool web). */
export const POOL_WEB_URL = "https://pool.vericonomy.com";
export const POOL_STRATUM_URL = "stratum+tcp://mine.vericonomy.com:3333";
/**
 * Optional comma-separated failover stratum URL(s) for the sidecar backend.
 * Empty by default (single endpoint); set when a backup pool node is available.
 */
export const POOL_STRATUM_BACKUP_URL = "";
export const POOL_FEE_PERCENT = 0;
export const POOL_PAYOUT_THRESHOLD_VRM = 1.0;
export const POOL_MIN_PAYOUT_CONFIRMATIONS = 120;
export const VERIUM_POOL_PAYOUT_ADDRESS = "VRq98Nm2P6anLHPgnHdb6NnibJ6GoG3Jm9";
export const VERIUM_POOL_DISPLAY_NAME = "Verium Pool";

/** Shown wherever the public pool is referenced in the wallet. */
export const POOL_DISCLAIMER =
  "This pool is provided as-is, without warranties. Mining, payouts, and displayed stats may be delayed, incorrect, or interrupted. You use the pool at your own risk; the operator is not liable for lost rewards, downtime, software errors, network issues, or any other damages.";

export function poolMinerUrl(address: string): string {
  return `${POOL_WEB_URL}/miner/${encodeURIComponent(address)}`;
}

export function poolWorkerUsername(
  address: string,
  worker = "desktop",
): string {
  return `${address}.${worker}`;
}
