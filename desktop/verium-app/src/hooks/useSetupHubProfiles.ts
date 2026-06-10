import { useQuery } from "@tanstack/react-query";
import { coinQueryKey, type CoinId } from "@/lib/coin/profile";
import { tauriWalletProfile, type WalletProfile } from "@/lib/wallet-profile";

/** Hub cards: one disk read per coin; invalidated after create/import/unlock. */
const HUB_PROFILE_STALE_MS = 60_000;
const HUB_PROFILE_GC_MS = 120_000;

export function useSetupHubProfile(coin: CoinId, enabled = true) {
  return useQuery<WalletProfile>({
    queryKey: coinQueryKey(coin, "wallet-profile"),
    queryFn: () => tauriWalletProfile(coin),
    enabled,
    staleTime: HUB_PROFILE_STALE_MS,
    gcTime: HUB_PROFILE_GC_MS,
    refetchOnMount: "always",
    refetchOnWindowFocus: false,
  });
}

/** Presence flags derived from a unified wallet profile. */
export function profileWalletPresence(profile: WalletProfile | undefined) {
  return {
    hasLightWallet: profile?.keys_present.light === true,
    hasFullNodeWallet:
      profile?.keys_present.full_node === true ||
      profile?.legacy.detected === true,
  };
}
