import type { QueryClient } from '@tanstack/react-query';
import { coinQueryKey, type CoinId } from '@/lib/coin/profile';
import { lightWalletRescan } from '@/lib/light-wallet/client';
import { rpcGetWalletInfo } from '@/lib/rpc/client';
import { isWalletLocked } from '@/lib/wallet-unlock';

/** Refresh home dashboard data after pull-to-refresh on mobile. */
export async function refreshMobileDashboard(
  queryClient: QueryClient,
  coin: CoinId,
  isLight: boolean
): Promise<void> {
  const wallet =
    queryClient.getQueryData<Awaited<ReturnType<typeof rpcGetWalletInfo>>>(
      coinQueryKey(coin, 'getwalletinfo')
    ) ??
    (await queryClient.fetchQuery({
      queryKey: coinQueryKey(coin, 'getwalletinfo'),
      queryFn: () => rpcGetWalletInfo(coin),
    }));

  const unlocked = wallet != null && !isWalletLocked(wallet);

  const invalidateTasks: Promise<unknown>[] = [
    queryClient.invalidateQueries({
      queryKey: coinQueryKey(coin, 'light-server-status'),
    }),
    queryClient.invalidateQueries({
      queryKey: coinQueryKey(coin, 'explorer-stats'),
    }),
    queryClient.invalidateQueries({
      predicate: (q) =>
        Array.isArray(q.queryKey) && q.queryKey[0] === coin && q.queryKey[1] === 'explorer-blocks',
    }),
    queryClient.invalidateQueries({
      queryKey: coinQueryKey(coin, 'getwalletinfo'),
    }),
    queryClient.invalidateQueries({
      queryKey: coinQueryKey(coin, 'wallet-cumulative-txs'),
    }),
  ];

  if (isLight && unlocked) {
    invalidateTasks.push(lightWalletRescan(coin));
  }

  await Promise.all(invalidateTasks);

  await Promise.all([
    queryClient.refetchQueries({
      queryKey: coinQueryKey(coin, 'light-server-status'),
    }),
    queryClient.refetchQueries({
      queryKey: coinQueryKey(coin, 'explorer-stats'),
    }),
    queryClient.refetchQueries({
      queryKey: coinQueryKey(coin, 'getwalletinfo'),
    }),
  ]);
}
