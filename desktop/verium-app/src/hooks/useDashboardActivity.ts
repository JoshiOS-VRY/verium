import { useQuery } from "@tanstack/react-query";
import type { CoinId } from "@/lib/coin/profile";
import { coinQueryKey } from "@/lib/coin/profile";
import { useNodeStatus } from "@/hooks/useNodeStatus";
import { useWindowVisible } from "@/hooks/useWindowVisible";
import { fetchExplorerStats } from "@/lib/explorer-api";
import { deriveDashboardActivity } from "@/lib/node/dashboard-activity";
import { useExplorerQueriesEnabled } from "@/lib/network-mode";
import { rpcGetBlockchainInfo } from "@/lib/rpc/client";

/**
 * Node + chain activity for the global banner — intentionally excludes mining
 * polls so the mining page does not stack hashrate RPC on every route.
 */
export function useDashboardActivity(coin: CoinId) {
  const visible = useWindowVisible();
  const explorerEnabled = useExplorerQueriesEnabled();
  const node = useNodeStatus(coin);

  const blockchain = useQuery({
    queryKey: coinQueryKey(coin, "getblockchaininfo"),
    queryFn: () => rpcGetBlockchainInfo(coin),
    refetchInterval: visible ? 30_000 : false,
    enabled: node.data?.connected === true,
  });

  const explorer = useQuery({
    queryKey: coinQueryKey(coin, "explorer-stats"),
    queryFn: () => fetchExplorerStats(coin),
    refetchInterval: visible ? 60_000 : false,
    enabled: explorerEnabled && node.data?.connected === true,
    retry: 0,
  });

  const activity = deriveDashboardActivity({
    coin,
    status: node.data,
    statusLoading: node.isLoading,
    isConnecting: node.isConnecting,
    blockchain: blockchain.data,
    blockchainLoading: blockchain.isLoading || blockchain.isFetching,
    networkTip: explorer.data?.height,
  });

  return {
    ...node,
    blockchain,
    explorer,
    activity,
  };
}
