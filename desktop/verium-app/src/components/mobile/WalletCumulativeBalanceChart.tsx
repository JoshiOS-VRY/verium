import { useEffect, useMemo } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import {
  CumulativeBalanceChart,
  CumulativeBalanceChartSkeleton,
} from '@/components/mobile/explorer/CumulativeBalanceChart';
import { useActiveCoin } from '@/lib/coin/context';
import { coinQueryKey } from '@/lib/coin/profile';
import {
  buildWalletCumulativeSeries,
  cumulativeSeriesCaption,
  indexerWalletSeriesToChart,
  type ChartCumulativePoint,
} from '@/lib/cumulative-balance';
import { fetchIndexerWalletCumulativeSeries } from '@/lib/indexer-api';
import { useExplorerQueriesEnabled } from '@/lib/network-mode';
import { rpcGetWalletInfo, rpcListTransactions, type WalletInfo } from '@/lib/rpc/client';
import { TRANSACTIONS_LIST_CAP } from '@/lib/transactions-list';
import { lockedWalletBalanceClass } from '@/lib/wallet-unlock';
import { useWalletMode } from '@/hooks/useWalletMode';

function isLightWalletChartSettling(wallet: WalletInfo): boolean {
  if (!wallet.light_wallet) return false;
  if (wallet.light_syncing) return true;
  if (wallet.light_balance_syncing) return true;
  if (wallet.light_scan_phase != null && wallet.light_scan_phase !== 'complete') return true;
  if (wallet.light_balance_ready === false) return true;
  return false;
}

function isChartAwaitingHistory(
  pointCount: number,
  anchorBalance: number,
  wallet: WalletInfo,
  indexerIncomplete: boolean
): boolean {
  if (anchorBalance <= 0 || pointCount > 1) return false;
  return isLightWalletChartSettling(wallet) || indexerIncomplete;
}

export function WalletCumulativeBalanceChart({
  embedded = false,
  blurClass: blurClassProp,
  onScrubChange,
  onPointsChange,
}: {
  embedded?: boolean;
  blurClass?: string;
  onScrubChange?: (point: ChartCumulativePoint | null, scrubbing: boolean) => void;
  onPointsChange?: (points: ChartCumulativePoint[]) => void;
}) {
  const coin = useActiveCoin();
  const { isLight } = useWalletMode();
  const explorerEnabled = useExplorerQueriesEnabled();
  const queryClient = useQueryClient();

  const wallet = useQuery({
    queryKey: coinQueryKey(coin, 'getwalletinfo'),
    queryFn: () => rpcGetWalletInfo(coin),
    refetchInterval: false,
  });

  const walletData = wallet.data;
  const anchorBalance =
    (walletData?.balance ?? 0) +
    (walletData?.unconfirmed_balance ?? 0) +
    (walletData?.immature_balance ?? 0);

  const useIndexerAggregate = isLight && explorerEnabled;

  const indexerSeries = useQuery({
    queryKey: coinQueryKey(coin, 'wallet-cumulative-indexer'),
    queryFn: () => fetchIndexerWalletCumulativeSeries(coin, anchorBalance),
    enabled: walletData != null && useIndexerAggregate,
    staleTime: 60_000,
  });

  const txs = useQuery({
    queryKey: coinQueryKey(coin, 'wallet-cumulative-txs'),
    queryFn: () => rpcListTransactions(coin, TRANSACTIONS_LIST_CAP, 0),
    enabled: walletData != null && (!useIndexerAggregate || indexerSeries.isError),
    staleTime: 60_000,
  });

  const walletSettling = walletData != null && isLightWalletChartSettling(walletData);

  useEffect(() => {
    if (!walletData || !useIndexerAggregate) return;
    void queryClient.invalidateQueries({
      queryKey: coinQueryKey(coin, 'wallet-cumulative-indexer'),
    });
  }, [
    coin,
    queryClient,
    useIndexerAggregate,
    walletData?.light_syncing,
    walletData?.light_balance_syncing,
    walletData?.light_balance_ready,
    walletData?.light_scan_phase,
    walletData?.private_keys_enabled,
  ]);

  useEffect(() => {
    if (!walletData || !useIndexerAggregate || !walletSettling) return;
    const id = window.setInterval(() => {
      void queryClient.invalidateQueries({
        queryKey: coinQueryKey(coin, 'wallet-cumulative-indexer'),
      });
    }, 8_000);
    return () => window.clearInterval(id);
  }, [coin, queryClient, useIndexerAggregate, walletData, walletSettling]);

  const useIndexerData = useIndexerAggregate && indexerSeries.data != null && !indexerSeries.isError;
  const historyLoading = useIndexerAggregate
    ? indexerSeries.isLoading && !indexerSeries.isError
    : txs.isLoading;

  const series = useMemo(() => {
    if (!walletData) return null;
    if (useIndexerData) {
      return indexerWalletSeriesToChart(indexerSeries.data!, anchorBalance);
    }
    return buildWalletCumulativeSeries(txs.data ?? [], anchorBalance);
  }, [walletData, useIndexerData, indexerSeries.data, txs.data, anchorBalance]);

  useEffect(() => {
    if (!series?.points.length || !onPointsChange) return;
    onPointsChange(series.points);
  }, [series, onPointsChange]);

  const indexerIncomplete =
    useIndexerData &&
    indexerSeries.data != null &&
    !indexerSeries.data.complete &&
    indexerSeries.data.txCountUsed === 0;

  const awaitingHistory =
    walletData != null &&
    series != null &&
    isChartAwaitingHistory(series.points.length, anchorBalance, walletData, indexerIncomplete);

  const showSkeleton =
    wallet.isLoading ||
    !walletData ||
    historyLoading ||
    awaitingHistory ||
    (useIndexerAggregate &&
      indexerSeries.isFetching &&
      series != null &&
      series.points.length <= 1 &&
      anchorBalance > 0);

  if (showSkeleton) {
    return <CumulativeBalanceChartSkeleton embedded={embedded} />;
  }

  if (!series) {
    return <CumulativeBalanceChartSkeleton embedded={embedded} />;
  }

  const blurClass = blurClassProp ?? lockedWalletBalanceClass(walletData);
  const ticker = coin === 'verium' ? 'VRM' : 'VRC';
  const captionSource = useIndexerData ? indexerSeries.data : null;

  return (
    <CumulativeBalanceChart
      points={series.points}
      ticker={ticker}
      coin={coin}
      anchorBalance={anchorBalance}
      title="Wallet balance history"
      caption={
        embedded
          ? undefined
          : cumulativeSeriesCaption(
              series.complete,
              series.txCountUsed,
              captionSource?.txCountTotal,
              useIndexerData,
              captionSource?.addressesTruncated
                ? {
                    truncated: true,
                    used: captionSource.addressCountUsed,
                    total: captionSource.addressCountTotal,
                  }
                : undefined
            )
      }
      blurClass={blurClass}
      embedded={embedded}
      onScrubChange={onScrubChange}
    />
  );
}
