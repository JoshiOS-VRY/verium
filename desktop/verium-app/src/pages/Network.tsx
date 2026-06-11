import { useState } from 'react';
import { useActiveCoin } from '@/lib/coin/context';
import { coinQueryKey, getCoinProfile } from '@/lib/coin/profile';
import { useQuery } from '@tanstack/react-query';
import { useWindowVisible } from '@/hooks/useWindowVisible';
import { NetworkHero } from '@/components/NetworkHero';
import { NetworkPeersSection } from '@/components/NetworkPeersSection';
import { NetworkTopMinersCard } from '@/components/NetworkTopMinersCard';
import { fetchExplorerExtraction, fetchExplorerStats } from '@/lib/explorer-api';
import type { MinersPeriodId } from '@/lib/miners-periods';
import { useExplorerQueriesEnabled } from '@/lib/network-mode';
import { isChainSynced, syncTargetHeight } from '@/lib/bootstrap-policy';
import {
  rpcGetBlockchainInfo,
  rpcGetMiningInfo,
  rpcGetNetworkInfo,
  rpcGetPeerInfo,
  rpcGetVericoinMiningInfo,
} from '@/lib/rpc/client';

export function Network() {
  const coin = useActiveCoin();
  const profile = getCoinProfile(coin);
  const visible = useWindowVisible();

  const network = useQuery({
    queryKey: coinQueryKey(coin, 'getnetworkinfo'),
    queryFn: () => rpcGetNetworkInfo(coin),
    refetchInterval: visible ? 5_000 : false,
  });
  const peers = useQuery({
    queryKey: coinQueryKey(coin, 'getpeerinfo'),
    queryFn: () => rpcGetPeerInfo(coin),
    refetchInterval: visible ? 5_000 : false,
  });
  const blockchain = useQuery({
    queryKey: coinQueryKey(coin, 'getblockchaininfo'),
    queryFn: () => rpcGetBlockchainInfo(coin),
    refetchInterval: false,
  });
  const vrmMining = useQuery({
    queryKey: coinQueryKey('verium', 'getmininginfo'),
    queryFn: () => rpcGetMiningInfo('verium'),
    enabled: visible && coin === 'verium',
    refetchInterval: false,
  });
  const vrcMining = useQuery({
    queryKey: coinQueryKey('vericoin', 'getmininginfo'),
    queryFn: () => rpcGetVericoinMiningInfo(),
    enabled: visible && coin === 'vericoin',
    refetchInterval: false,
  });

  const explorerEnabled = useExplorerQueriesEnabled();

  const [minersPeriod, setMinersPeriod] = useState<MinersPeriodId>('month');

  const extraction = useQuery({
    queryKey: coinQueryKey(coin, 'explorer-extraction', minersPeriod),
    queryFn: () => fetchExplorerExtraction(coin, 15, minersPeriod),
    enabled: explorerEnabled && coin === 'verium',
    refetchInterval: visible ? 60_000 : false,
    retry: 0,
  });

  const explorerStats = useQuery({
    queryKey: coinQueryKey(coin, 'explorer-stats'),
    queryFn: () => fetchExplorerStats(coin),
    enabled: explorerEnabled,
    refetchInterval: false,
    retry: 0,
  });

  const localHeight = blockchain.data?.blocks;
  const headerHeight = blockchain.data?.headers ?? blockchain.data?.blocks ?? 0;
  const networkTip = syncTargetHeight(blockchain.data, explorerStats.data?.height);
  const peerCount = Math.max(network.data?.connections ?? 0, peers.data?.length ?? 0);
  const chainSynced = isChainSynced(blockchain.data, {
    connected: network.data != null || peers.data != null,
    syncStalled: false,
    networkTip: explorerStats.data?.height,
  });

  return (
    <div className="flex flex-col gap-5 sm:gap-4">
      <NetworkHero
        profile={profile}
        coin={coin}
        localBlocks={localHeight}
        headerHeight={headerHeight}
        networkTip={networkTip}
        peerCount={peerCount}
        network={network.data}
        blockchain={blockchain.data}
        explorer={explorerStats.data}
        explorerError={explorerStats.isError}
        localHashrate={vrmMining.data?.hashrate}
        vrcMining={vrcMining.data}
        networkActive={network.data?.networkactive}
        chainSynced={chainSynced}
        ibd={blockchain.data?.initialblockdownload}
      />

      {explorerEnabled && coin === 'verium' && (
        <NetworkTopMinersCard
          coin={coin}
          period={minersPeriod}
          onPeriodChange={setMinersPeriod}
          entries={extraction.data}
          isError={extraction.isError}
          isLoading={extraction.isLoading}
          isFetching={extraction.isFetching}
        />
      )}

      <NetworkPeersSection coin={coin} peers={peers.data} explorerEnabled={explorerEnabled} />
    </div>
  );
}
