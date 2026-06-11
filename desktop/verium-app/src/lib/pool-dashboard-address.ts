import type { UserPreferences } from '@/lib/user-preferences';
import { staticMiningAddressConfigured } from '@/lib/mining-reward-address';

export function poolPayoutAddressConfigured(
  prefs: Pick<UserPreferences, 'pool_payout_address'>
): boolean {
  return Boolean(prefs.pool_payout_address?.trim());
}

/** VRM address used for pool Stratum username and in-wallet pool dashboard. */
export function resolvePoolDashboardAddress(
  prefs: Pick<
    UserPreferences,
    'pool_payout_address' | 'mining_reward_address_mode' | 'mining_reward_address'
  >,
  addresses?: string[]
): string | undefined {
  const explicit = prefs.pool_payout_address?.trim();
  if (explicit) {
    return explicit;
  }
  if (staticMiningAddressConfigured(prefs)) {
    return prefs.mining_reward_address?.trim();
  }
  const first = addresses?.find((a) => a?.trim().startsWith('V'));
  return first?.trim();
}

/** Default pool payout when user has not chosen one yet. */
export function suggestPoolPayoutAddress(
  prefs: Pick<UserPreferences, 'mining_reward_address_mode' | 'mining_reward_address'>,
  addresses?: string[]
): string | undefined {
  if (staticMiningAddressConfigured(prefs)) {
    return prefs.mining_reward_address?.trim();
  }
  const first = addresses?.find((a) => a?.trim().startsWith('V'));
  return first?.trim();
}
