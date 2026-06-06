/** Official Verium pool constants (aligned with explorer + pool web). */
export const POOL_WEB_URL = "https://pool.vericonomy.com";
export const POOL_STRATUM_URL = "stratum+tcp://mine.vericonomy.com:3333";
export const POOL_FEE_PERCENT = 1;
export const POOL_PAYOUT_THRESHOLD_VRM = 1.0;
export const POOL_MIN_PAYOUT_CONFIRMATIONS = 120;
export const VERIUM_POOL_PAYOUT_ADDRESS = "VRq98Nm2P6anLHPgnHdb6NnibJ6GoG3Jm9";
export const VERIUM_POOL_DISPLAY_NAME = "Verium Pool";

export function poolMinerUrl(address: string): string {
  return `${POOL_WEB_URL}/miner/${encodeURIComponent(address)}`;
}

export function poolWorkerUsername(address: string, worker = "desktop"): string {
  return `${address}.${worker}`;
}
