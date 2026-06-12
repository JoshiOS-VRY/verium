import { useEffect } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import type { CoinId } from '@/lib/coin/profile';
import { coinQueryKey } from '@/lib/coin/profile';
import { lightWalletRefreshPending } from '@/lib/light-wallet/client';
import { walletTransactionsKeyPrefix } from '@/lib/wallet-transactions-query';
import { useWindowVisible } from '@/hooks/useWindowVisible';

const RECEIVE_POLL_MS = 15_000;

/** Poll Electrum for pending UTXOs/history while the receive screen is open. */
export function useLightWalletInstantReceiveSync(coin: CoinId, enabled: boolean) {
  const visible = useWindowVisible();
  const queryClient = useQueryClient();

  useEffect(() => {
    if (!enabled || !visible) return;

    let cancelled = false;

    const tick = async () => {
      try {
        await lightWalletRefreshPending(coin);
        if (cancelled) return;
        await queryClient.invalidateQueries({ queryKey: walletTransactionsKeyPrefix(coin) });
        await queryClient.invalidateQueries({ queryKey: coinQueryKey(coin, 'getwalletinfo') });
      } catch {
        // Best-effort foreground refresh; steady sync still runs in the background.
      }
    };

    void tick();
    const id = window.setInterval(() => void tick(), RECEIVE_POLL_MS);
    return () => {
      cancelled = true;
      window.clearInterval(id);
    };
  }, [coin, enabled, queryClient, visible]);
}
