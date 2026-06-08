import { useQuery } from "@tanstack/react-query";
import { coinQueryKey, type CoinId } from "@/lib/coin/profile";
import { tauriWalletProfile, type WalletProfile } from "@/lib/wallet-profile";

/**
 * Unified per-coin readiness. Use `profile.ready` to gate dashboard access and
 * `profile.intent` to route the setup wizard. Refetches on mount so the hub and
 * guards reflect freshly-created/imported wallets without a manual reload.
 */
export function useWalletProfile(coin: CoinId, enabled = true) {
  return useQuery<WalletProfile>({
    queryKey: coinQueryKey(coin, "wallet-profile"),
    queryFn: () => tauriWalletProfile(coin),
    enabled,
    staleTime: 0,
    refetchOnMount: "always",
  });
}
