import { useQuery } from "@tanstack/react-query";
import { coinQueryKey } from "@/lib/coin/profile";
import {
  MINER_STATE_IDLE_PROBE_MS,
  MINER_STATE_POLL_MS,
  MINING_INFO_POLL_MS,
  POOL_MINER_IDLE_PROBE_MS,
  POOL_MINER_STATUS_POLL_MS,
} from "@/lib/mining-poll";
import { fetchPoolMinerStatus } from "@/lib/pool-miner-api";
import { useUserPreferences } from "@/lib/user-preferences";
import { rpcGetMinerState, rpcGetMiningInfo } from "@/lib/rpc/client";
import { useWindowVisible } from "@/hooks/useWindowVisible";

const VERIUM = "verium" as const;

/**
 * Single writer for mining-related React Query polls.
 *
 * Other hooks/components must use `refetchInterval: false` on the same keys or
 * React Query will use the minimum interval across observers (e.g. 4s × N
 * components → WebView OOM while mining).
 */
export function useMiningPollCoordinator(): void {
  const visible = useWindowVisible();
  const veriumEnabled = useUserPreferences((s) => s.prefs.verium_enabled !== false);

  useQuery({
    queryKey: ["pool-miner", "status"],
    queryFn: fetchPoolMinerStatus,
    enabled: veriumEnabled,
    refetchInterval: (query) => {
      if (!visible) return false;
      return query.state.data?.running
        ? POOL_MINER_STATUS_POLL_MS
        : POOL_MINER_IDLE_PROBE_MS;
    },
    staleTime: 10_000,
    gcTime: 30_000,
  });

  const miner = useQuery({
    queryKey: coinQueryKey(VERIUM, "get_miner_state"),
    queryFn: () => rpcGetMinerState(VERIUM),
    enabled: veriumEnabled,
    refetchInterval: (query) => {
      if (!visible) return false;
      return query.state.data?.active
        ? MINER_STATE_POLL_MS
        : MINER_STATE_IDLE_PROBE_MS;
    },
    staleTime: 10_000,
    gcTime: 30_000,
  });

  const soloActive = miner.data?.active ?? false;

  useQuery({
    queryKey: coinQueryKey(VERIUM, "getmininginfo"),
    queryFn: () => rpcGetMiningInfo(VERIUM),
    enabled: veriumEnabled && soloActive,
    refetchInterval: visible && soloActive ? MINING_INFO_POLL_MS : false,
    staleTime: 10_000,
    gcTime: 30_000,
  });
}
