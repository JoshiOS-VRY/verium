import { useEffect, useMemo, useRef } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { coinQueryKey, type CoinId } from "@/lib/coin/profile";
import {
  isBinaryUnavailableError,
  isDaemonConnectingState,
} from "@/lib/daemon-connecting";
import { subscribeNodeStateChanged } from "@/lib/node-state-listener";
import { nodeStateFromStatus } from "@/lib/node/status";
import { rpcGetNodeStatus, type NodeStatus } from "@/lib/rpc/client";

/** How long to treat unreachable RPC as "still starting" after app open. */
const STARTUP_GRACE_MS = 120_000;

export function useNodeStatus(coin: CoinId) {
  const mountedAt = useRef(Date.now());
  const queryClient = useQueryClient();

  const query = useQuery<NodeStatus>({
    queryKey: coinQueryKey(coin, "daemon-status"),
    queryFn: () => rpcGetNodeStatus(coin),
    refetchInterval: (q) => {
      const d = q.state.data;
      if (isBinaryUnavailableError(d?.error)) return false;
      if (d?.warming_up || d?.reindex_in_progress || d?.sync_stalled) return 10_000;
      if (d?.connected) return 20_000;
      return 10_000;
    },
    retry: 1,
    retryDelay: 2_000,
  });

  // One Tauri listener per coin (shared across all hook instances).
  useEffect(() => {
    const queryKey = coinQueryKey(coin, "daemon-status");
    return subscribeNodeStateChanged(coin, () => {
      void queryClient.invalidateQueries({ queryKey });
    });
  }, [coin, queryClient]);

  const startupGraceActive = Date.now() - mountedAt.current < STARTUP_GRACE_MS;

  const isConnecting = useMemo(
    () =>
      isDaemonConnectingState(query.data, {
        isLoading: query.isLoading,
        isFetching: query.isFetching,
        startupGraceActive,
      }),
    [query.data, query.isLoading, query.isFetching, startupGraceActive],
  );

  const nodeState = nodeStateFromStatus(query.data);

  return { ...query, isConnecting, nodeState };
}

/** @deprecated Use useNodeStatus — kept for gradual migration. */
export const useDaemonStatus = useNodeStatus;

export function resetDaemonEnsureAttempt(_coin?: CoinId) {
  // No-op: backend orchestrator owns auto-start now.
}
