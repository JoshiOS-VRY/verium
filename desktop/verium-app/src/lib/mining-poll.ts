/** Intervals for the single app-wide mining poll coordinator (see useMiningPollCoordinator). */

/** Pool sidecar / native status — one poll for all UI observers. */
export const POOL_MINER_STATUS_POLL_MS = 15_000;

/** Solo miner active flag + thread count. */
export const MINER_STATE_POLL_MS = 15_000;

/** Local hashrate via `getmininginfo` — solo CPU mining only (not pool sidecar). */
export const MINING_INFO_POLL_MS = 20_000;

/** Pool Supabase dashboard (overview, history, payouts) when not locally hashing. */
export const POOL_DASHBOARD_POLL_MS = 60_000;

/** Idle probe for pool miner (wallet open, miner off). */
export const POOL_MINER_IDLE_PROBE_MS = 60_000;

/** Idle probe for solo miner state when not known to be active. */
export const MINER_STATE_IDLE_PROBE_MS = 45_000;
