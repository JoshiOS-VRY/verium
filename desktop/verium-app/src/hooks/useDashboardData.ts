import { useQuery } from "@tanstack/react-query";
import { coinQueryKey, type CoinId } from "@/lib/coin/profile";
import { lightWalletExists } from "@/lib/light-wallet/client";
import { useBlockAgeTick } from "@/hooks/useBlockAgeTick";
import { useLightServerConnected } from "@/hooks/useLightServerConnected";
import { useNodeStatus } from "@/hooks/useNodeStatus";
import { useWalletMode } from "@/hooks/useWalletMode";
import { useWalletTransactions } from "@/hooks/useWalletTransactions";
import { useWindowVisible } from "@/hooks/useWindowVisible";
import {
  blocksBehindNetwork,
  chainSyncPhase,
  syncTargetHeight,
} from "@/lib/bootstrap-policy";
import { useChainTip } from "@/lib/chain-tip-store";
import { fetchExplorerBlocks, fetchExplorerStats } from "@/lib/explorer-api";
import {
  deriveDashboardActivity,
  type DashboardActivity,
} from "@/lib/node/dashboard-activity";
import { useExplorerQueriesEnabled } from "@/lib/network-mode";
import { fetchPoolMinerStatus } from "@/lib/pool-miner-api";
import {
  rpcGetBlockchainInfo,
  rpcGetMinerState,
  rpcGetMiningInfo,
  rpcGetStakingState,
  rpcGetVericoinMiningInfo,
  rpcGetWalletInfo,
  rpcRaw,
} from "@/lib/rpc/client";
import { resolveTipBlockTime } from "@/lib/tip-block-time";
import { formatBlockAge } from "@/lib/utils";
import { walletInfoForMode } from "@/lib/wallet-unlock";

/** Shared RPC polling for dashboard hero, middle row, and activity banners. */
export function useDashboardData(coin: CoinId) {
  const visible = useWindowVisible();
  const explorerEnabled = useExplorerQueriesEnabled();
  const { isLight } = useWalletMode();
  const lightExistsForCoin = useQuery({
    queryKey: coinQueryKey(coin, "light-wallet-exists"),
    queryFn: () => lightWalletExists(coin),
    enabled: isLight,
    staleTime: 0,
    refetchOnMount: "always",
  });
  const lightServer = useLightServerConnected();
  const node = useNodeStatus(coin);
  const chainTip = useChainTip(coin);
  const ageTick = useBlockAgeTick(visible);

  const connected = isLight
    ? lightServer.connected
    : node.data?.connected === true;

  const blockchain = useQuery({
    queryKey: coinQueryKey(coin, "getblockchaininfo"),
    queryFn: () => rpcGetBlockchainInfo(coin),
    refetchInterval: false,
    enabled: !isLight,
  });

  const wallet = useQuery({
    queryKey: coinQueryKey(coin, "getwalletinfo"),
    queryFn: () => rpcGetWalletInfo(coin),
    enabled: !isLight || lightExistsForCoin.data !== false,
    refetchInterval: (q) => {
      if (!isLight || !visible || !lightServer.connected) return false;
      const effective = walletInfoForMode(true, q.state.data ?? undefined);
      if (!effective) return false;
      if (effective.light_syncing) return 5_000;
      return 30_000;
    },
    retry: isLight ? 2 : 3,
  });

  const effectiveWallet = walletInfoForMode(isLight, wallet.data);

  const explorer = useQuery({
    queryKey: coinQueryKey(coin, "explorer-stats"),
    queryFn: () => fetchExplorerStats(coin),
    refetchInterval: visible ? 30_000 : false,
    enabled: explorerEnabled && connected,
    retry: 0,
  });

  const explorerBlocks = useQuery({
    queryKey: coinQueryKey(coin, "explorer-blocks", 10),
    queryFn: () => fetchExplorerBlocks(coin, 10),
    enabled: explorerEnabled && connected && visible,
    staleTime: isLight ? 5_000 : 60_000,
    refetchInterval: visible ? (isLight ? 5_000 : 60_000) : false,
    retry: 2,
  });

  const transactions = useWalletTransactions(coin, {
    enabled: connected && (!isLight || lightExistsForCoin.data !== false),
  });

  const minerState = useQuery({
    queryKey: coinQueryKey(coin, "get_miner_state"),
    queryFn: () => rpcGetMinerState(coin),
    refetchInterval: false,
    enabled: coin === "verium" && !isLight,
  });

  const minerActive = minerState.data?.active ?? false;
  const minerStartedAt = minerState.data?.started_at;

  const poolMiner = useQuery({
    queryKey: ["pool-miner", "status"],
    queryFn: fetchPoolMinerStatus,
    refetchInterval: false,
    gcTime: 30_000,
    enabled: coin === "verium" && connected && !isLight,
  });

  const poolMinerRunning = poolMiner.data?.running ?? false;
  const poolHashrate = poolMiner.data?.hashrateHm ?? 0;
  const miningActive = minerActive || poolMinerRunning;

  const mining = useQuery({
    queryKey: coinQueryKey(coin, "getmininginfo"),
    queryFn: () => rpcGetMiningInfo(coin),
    refetchInterval: false,
    enabled: coin === "verium" && !isLight && miningActive,
  });

  const soloHashrate = mining.data?.hashrate ?? 0;
  const localHashrate = poolMinerRunning ? poolHashrate : soloHashrate;

  const stakingState = useQuery({
    queryKey: coinQueryKey(coin, "get_staking_state"),
    queryFn: () => rpcGetStakingState(coin),
    refetchInterval: false,
    enabled: coin === "vericoin" && !isLight,
  });

  const vrcMining = useQuery({
    queryKey: coinQueryKey("vericoin", "getmininginfo"),
    queryFn: () => rpcGetVericoinMiningInfo(),
    refetchInterval: false,
    enabled: coin === "vericoin" && !isLight,
  });

  const tipHashForHeader =
    chainTip.tip?.hash ?? blockchain.data?.bestblockhash ?? "";

  const tipHeaderTime = useQuery({
    queryKey: coinQueryKey(coin, "blockheader-time", tipHashForHeader),
    queryFn: async () => {
      const header = (await rpcRaw(coin, "getblockheader", [
        tipHashForHeader,
      ])) as { time?: number };
      return header.time != null && header.time > 0 ? header.time : null;
    },
    enabled: !isLight && Boolean(tipHashForHeader),
    staleTime: 15_000,
    refetchInterval: visible && !isLight ? 30_000 : false,
  });

  const networkTip = explorer.data?.height ?? lightServer.tipHeight ?? undefined;
  const syncCtx = {
    connected,
    syncStalled: isLight ? false : node.data?.sync_stalled === true,
    networkTip,
  };
  const phase = isLight ? "synced" : chainSyncPhase(blockchain.data, syncCtx);
  const synced = isLight || phase === "synced";
  const localBlocks = isLight
    ? (lightServer.tipHeight ?? explorer.data?.height)
    : blockchain.data?.blocks;
  const blockHash = blockchain.data?.bestblockhash;
  const tipHeight =
    (isLight ? lightServer.tipHeight : null) ??
    chainTip.tip?.height ??
    localBlocks;
  const tipHash = chainTip.tip?.hash ?? blockHash;
  const syncTarget = syncTargetHeight(blockchain.data, networkTip);
  const behind = blocksBehindNetwork(localBlocks, syncTarget);

  const tipTime = resolveTipBlockTime(tipHeight, {
    chainTip: chainTip.tip,
    explorerBlocks: explorerBlocks.data,
    headerTime: tipHeaderTime.data,
  });
  const blockAge = tipTime != null ? formatBlockAge(tipTime, ageTick) : "—";
  const connections = isLight ? 0 : (node.data?.connections ?? 0);

  const activity: DashboardActivity = isLight
    ? {
        kind: connected ? "ready" : "unavailable",
        title: connected ? "Light wallet online" : "Light wallet offline",
        detail: connected
          ? "Connected to Vericonomy servers"
          : "Cannot reach Electrum servers",
        showSpinner: false,
      }
    : deriveDashboardActivity({
        coin,
        status: node.data,
        statusLoading: node.isLoading,
        isConnecting: node.isConnecting,
        blockchain: blockchain.data,
        blockchainLoading: blockchain.isLoading || blockchain.isFetching,
        networkTip,
      });

  return {
    isLight,
    lightServer,
    node,
    status: node.data,
    activity,
    connected,
    synced,
    phase,
    blockchain,
    wallet,
    effectiveWallet,
    explorer,
    transactions,
    mining,
    minerState,
    stakingState,
    vrcMining,
    chainTip,
    localBlocks,
    tipHeight,
    tipHash,
    syncTarget,
    behind,
    blockHash,
    blockAge,
    connections,
    networkTip,
    minerActive,
    minerStartedAt,
    poolMiner,
    poolMinerRunning,
    miningActive,
    localHashrate,
  };
}
