import { useQuery } from "@tanstack/react-query";
import { coinQueryKey } from "@/lib/coin/profile";
import { useUserPreferences } from "@/lib/user-preferences";
import { useWalletMode } from "@/hooks/useWalletMode";
import { useWindowVisible } from "@/hooks/useWindowVisible";
import { rpcGetStakingState, rpcGetVericoinMiningInfo } from "@/lib/rpc/client";

const STAKING_STATE_POLL_MS = 15_000;
const VRC_MINING_INFO_POLL_MS = 30_000;

/**
 * Single writer for Vericoin earn RPC — other hooks must use `refetchInterval: false`.
 */
export function useVericoinEarnPollCoordinator(): void {
  const visible = useWindowVisible();
  const { isLight } = useWalletMode();
  const vericoinEnabled = useUserPreferences((s) => s.prefs.vericoin_enabled !== false);
  const enabled = !isLight && vericoinEnabled;

  useQuery({
    queryKey: coinQueryKey("vericoin", "get_staking_state"),
    queryFn: () => rpcGetStakingState("vericoin"),
    enabled,
    refetchInterval: visible ? STAKING_STATE_POLL_MS : false,
    staleTime: 10_000,
    gcTime: 30_000,
  });

  useQuery({
    queryKey: coinQueryKey("vericoin", "getmininginfo"),
    queryFn: () => rpcGetVericoinMiningInfo(),
    enabled,
    refetchInterval: visible ? VRC_MINING_INFO_POLL_MS : false,
    staleTime: 10_000,
    gcTime: 30_000,
  });
}
