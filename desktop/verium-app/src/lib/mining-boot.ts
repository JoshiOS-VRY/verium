/** Poll interval for `getmininginfo` hashrate across the wallet UI. */
export const MINING_HASHRATE_POLL_MS = 10_000;

/** How long to show "Starting…" after minerstart before treating 0 H/m as stalled. */
export const MINER_BOOT_GRACE_SECONDS = 120;

export function isMinerBooting(
  active: boolean,
  hashrate: number,
  startedAt: number | undefined,
  startPending = false,
  stopPending = false,
): boolean {
  if (stopPending) return false;
  if (startPending) return true;
  if (!active || hashrate > 0) return false;
  if (!startedAt) return true;
  return Date.now() / 1000 - startedAt < MINER_BOOT_GRACE_SECONDS;
}

/** React Query refetch interval for local mining hashrate (`getmininginfo`). */
export function miningInfoRefetchMs(
  active?: boolean,
  hashrate?: number,
  startedAt?: number,
): number | false {
  if (!active) return false;
  if (hashrate !== undefined && hashrate > 0) return MINING_HASHRATE_POLL_MS;
  if (!startedAt) return MINING_HASHRATE_POLL_MS;
  if (Date.now() / 1000 - startedAt < MINER_BOOT_GRACE_SECONDS) {
    return MINING_HASHRATE_POLL_MS;
  }
  // Active but zero hashrate past boot grace — slow poll only.
  return MINING_HASHRATE_POLL_MS * 3;
}
