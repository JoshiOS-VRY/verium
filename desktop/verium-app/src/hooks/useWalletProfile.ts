import { useQuery } from '@tanstack/react-query';
import { coinQueryKey, type CoinId } from '@/lib/coin/profile';
import { tauriWalletProfile, type WalletProfile } from '@/lib/wallet-profile';

/** Default cache for readiness guards — avoid refetch storms on route/coin churn. */
const PROFILE_STALE_MS = 60_000;
const PROFILE_GC_MS = 120_000;

/**
 * Unified per-coin readiness. Use `profile.ready` to gate dashboard access and
 * `profile.intent` to route the setup wizard.
 */
export function useWalletProfile(coin: CoinId, enabled = true) {
  return useQuery<WalletProfile>({
    queryKey: coinQueryKey(coin, 'wallet-profile'),
    queryFn: () => tauriWalletProfile(coin),
    enabled,
    staleTime: PROFILE_STALE_MS,
    gcTime: PROFILE_GC_MS,
    refetchOnMount: false,
    refetchOnWindowFocus: false,
  });
}

/** Setup hub / post-import: force a fresh disk read once on mount. */
export function useWalletProfileFresh(coin: CoinId, enabled = true) {
  return useQuery<WalletProfile>({
    queryKey: coinQueryKey(coin, 'wallet-profile'),
    queryFn: () => tauriWalletProfile(coin),
    enabled,
    staleTime: 0,
    gcTime: PROFILE_GC_MS,
    refetchOnMount: 'always',
    refetchOnWindowFocus: false,
  });
}
