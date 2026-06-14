import { useQuery } from '@tanstack/react-query';
import type { CoinId } from '@/lib/coin/profile';
import {
  LIGHT_INCOMING_NOTIFY_BACKGROUND_POLL_MS,
  LIGHT_INCOMING_NOTIFY_POLL_MS,
  LIGHT_TX_BACKGROUND_POLL_MS,
  LIGHT_TX_POLL_MS,
} from '@/lib/light-wallet/poll';
import {
  fetchWalletTransactions,
  WALLET_TX_BACKGROUND_POLL_MS,
  WALLET_TX_POLL_INTERVAL_MS,
  walletTransactionsQueryKey,
} from '@/lib/wallet-transactions-query';
import { useCoinWalletMode } from '@/hooks/useWalletMode';
import { useWindowVisible } from '@/hooks/useWindowVisible';

export function useWalletTransactions(
  coin: CoinId,
  options?: { enabled?: boolean; incomingWatch?: boolean }
) {
  const visible = useWindowVisible();
  const { isLight } = useCoinWalletMode(coin);
  const incomingWatch = options?.incomingWatch === true && isLight;

  const refetchInterval = incomingWatch
    ? visible
      ? LIGHT_INCOMING_NOTIFY_POLL_MS
      : LIGHT_INCOMING_NOTIFY_BACKGROUND_POLL_MS
    : visible
      ? isLight
        ? LIGHT_TX_POLL_MS
        : WALLET_TX_POLL_INTERVAL_MS
      : isLight
        ? LIGHT_TX_BACKGROUND_POLL_MS
        : WALLET_TX_BACKGROUND_POLL_MS;

  return useQuery({
    queryKey: walletTransactionsQueryKey(coin),
    queryFn: () => fetchWalletTransactions(coin, { lightRefreshPending: incomingWatch || isLight }),
    refetchInterval,
    enabled: options?.enabled ?? true,
    retry: 0,
    gcTime: 30_000,
  });
}
