import { useEffect } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import type { CoinId } from '@/lib/coin/profile';
import { lightWalletRefreshPending } from '@/lib/light-wallet/client';
import { LIGHT_FOREGROUND_SYNC_MS } from '@/lib/light-wallet/poll';
import { walletTransactionsKeyPrefix } from '@/lib/wallet-transactions-query';
import { useWindowVisible } from '@/hooks/useWindowVisible';

/** Foreground Electrum refresh while Activity (send/receive/history) is open. */
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
      } catch {
        // Best-effort foreground refresh; steady sync still runs in the background.
      }
    };

    void tick();
    const id = window.setInterval(() => void tick(), LIGHT_FOREGROUND_SYNC_MS);
    return () => {
      cancelled = true;
      window.clearInterval(id);
    };
  }, [coin, enabled, queryClient, visible]);
}
