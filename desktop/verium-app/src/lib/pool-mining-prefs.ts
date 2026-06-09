import type { UserPreferences } from "@/lib/user-preferences";

/** Normalize worker label for Stratum `ADDRESS.worker` usernames. */
export function normalizePoolWorkerName(name: string): string {
  const trimmed = name.trim();
  return trimmed || "wallet";
}

export function normalizePoolPayoutAddress(address: string): string {
  return address.trim();
}

/** Patch written to disk so pool identity survives wallet restarts. */
export function poolMiningPrefsPatch(
  payoutAddress: string,
  workerName: string,
): Pick<UserPreferences, "pool_payout_address" | "pool_worker_name"> {
  return {
    pool_payout_address: normalizePoolPayoutAddress(payoutAddress),
    pool_worker_name: normalizePoolWorkerName(workerName),
  };
}
