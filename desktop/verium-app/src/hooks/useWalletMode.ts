import { useQuery, useQueryClient } from "@tanstack/react-query";
import { LIGHT_WALLET_ENABLED } from "@/lib/features";
import { walletModeGet } from "@/lib/light-wallet/client";

export const WALLET_MODE_QUERY_KEY = ["wallet-mode-status"] as const;

export function useWalletMode() {
  const { data, isLoading } = useQuery({
    queryKey: WALLET_MODE_QUERY_KEY,
    queryFn: walletModeGet,
    staleTime: 30_000,
  });

  const lightWalletEnabled =
    LIGHT_WALLET_ENABLED || data?.light_wallet_enabled === true;
  const isLight = lightWalletEnabled && data?.mode === "light";

  return {
    isLoading,
    isLight,
    isFullNode: !isLight,
    mode: data?.mode ?? "full_node",
    lightWalletEnabled,
    lightWalletExists: data?.light_wallet_exists ?? false,
    electrumServers: data?.electrum_servers ?? [],
  };
}

export function useInvalidateWalletMode() {
  const queryClient = useQueryClient();
  return () => {
    void queryClient.invalidateQueries({ queryKey: WALLET_MODE_QUERY_KEY });
  };
}
