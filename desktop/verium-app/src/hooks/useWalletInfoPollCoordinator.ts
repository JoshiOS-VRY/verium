import { useQuery } from "@tanstack/react-query";
import { coinQueryKey } from "@/lib/coin/profile";
import { useWindowVisible } from "@/hooks/useWindowVisible";
import { useUserPreferences } from "@/lib/user-preferences";
import { useWalletMode } from "@/hooks/useWalletMode";
import { lightWalletExists } from "@/lib/light-wallet/client";
import { rpcGetWalletInfo, type WalletInfo } from "@/lib/rpc/client";

function walletScanProgress(
  scanning: WalletInfo["scanning"],
): { duration: number; progress: number } | null {
  return typeof scanning === "object" ? scanning : null;
}

const WALLET_INFO_POLL_MS = 30_000;
const WALLET_SCAN_POLL_MS = 5_000;

/**
 * Single writer for `getwalletinfo` — other hooks must use `refetchInterval: false`.
 */
export function useWalletInfoPollCoordinator(): void {
  const visible = useWindowVisible();
  const prefs = useUserPreferences((s) => s.prefs);
  const { isLight } = useWalletMode();
  const veriumLight = useQuery({
    queryKey: coinQueryKey("verium", "light-wallet-exists"),
    queryFn: () => lightWalletExists("verium"),
    enabled: prefs.verium_enabled !== false,
    staleTime: 30_000,
  });
  const vericoinLight = useQuery({
    queryKey: coinQueryKey("vericoin", "light-wallet-exists"),
    queryFn: () => lightWalletExists("vericoin"),
    enabled: prefs.vericoin_enabled !== false,
    staleTime: 30_000,
  });

  const interval = (data: WalletInfo | undefined) => {
    if (!visible) return false;
    if (data?.light_syncing) return WALLET_SCAN_POLL_MS;
    return walletScanProgress(data?.scanning)
      ? WALLET_SCAN_POLL_MS
      : WALLET_INFO_POLL_MS;
  };

  useQuery({
    queryKey: coinQueryKey("verium", "getwalletinfo"),
    queryFn: () => rpcGetWalletInfo("verium"),
    enabled: prefs.verium_enabled !== false && (!isLight || veriumLight.data !== false),
    refetchInterval: (q) => interval(q.state.data ?? undefined),
    staleTime: 10_000,
    gcTime: 30_000,
  });

  useQuery({
    queryKey: coinQueryKey("vericoin", "getwalletinfo"),
    queryFn: () => rpcGetWalletInfo("vericoin"),
    enabled:
      prefs.vericoin_enabled !== false &&
      (!isLight || vericoinLight.data !== false),
    refetchInterval: (q) => interval(q.state.data ?? undefined),
    staleTime: 10_000,
    gcTime: 30_000,
  });
}
