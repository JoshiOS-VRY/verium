import { useQuery } from '@tanstack/react-query';
import { CumulativeBalanceChart } from '@/components/mobile/explorer/CumulativeBalanceChart';
import { useActiveCoin } from '@/lib/coin/context';
import { coinQueryKey } from '@/lib/coin/profile';
import { buildWalletCumulativeSeries, cumulativeSeriesCaption } from '@/lib/cumulative-balance';
import { rpcGetWalletInfo, rpcListTransactions } from '@/lib/rpc/client';
import { TRANSACTIONS_LIST_CAP } from '@/lib/transactions-list';
import { lockedWalletBalanceClass } from '@/lib/wallet-unlock';

export function WalletCumulativeBalanceChart() {
  const coin = useActiveCoin();

  const wallet = useQuery({
    queryKey: coinQueryKey(coin, 'getwalletinfo'),
    queryFn: () => rpcGetWalletInfo(coin),
    refetchInterval: false,
  });

  const txs = useQuery({
    queryKey: coinQueryKey(coin, 'wallet-cumulative-txs'),
    queryFn: () => rpcListTransactions(coin, TRANSACTIONS_LIST_CAP, 0),
    enabled: wallet.data != null,
    staleTime: 60_000,
  });

  if (wallet.isLoading || txs.isLoading || !wallet.data) {
    return null;
  }

  const blurClass = lockedWalletBalanceClass(wallet.data);
  const anchorBalance =
    wallet.data.balance + wallet.data.unconfirmed_balance + wallet.data.immature_balance;
  const series = buildWalletCumulativeSeries(txs.data ?? [], anchorBalance);
  const ticker = coin === 'verium' ? 'VRM' : 'VRC';

  return (
    <CumulativeBalanceChart
      points={series.points}
      ticker={ticker}
      coin={coin}
      anchorBalance={anchorBalance}
      title="Wallet balance history"
      caption={cumulativeSeriesCaption(series.complete, series.txCountUsed)}
      blurClass={blurClass}
    />
  );
}
