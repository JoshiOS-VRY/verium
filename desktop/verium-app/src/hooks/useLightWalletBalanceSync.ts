import { useEffect } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { useActiveCoin } from '@/lib/coin/context';
import { coinQueryKey } from '@/lib/coin/profile';
import { lightWalletRefreshBalance } from '@/lib/light-wallet/client';
import { LIGHT_BALANCE_POLL_MS } from '@/lib/light-wallet/poll';
import { useCoinWalletMode } from '@/hooks/useWalletMode';
import { useWindowVisible } from '@/hooks/useWindowVisible';

/**
 * Foreground balance sync every ~15s: probe HD addresses, refresh UTXO cache, refetch wallet info.
 */
export function useLightWalletBalanceSync() {
  const coin = useActiveCoin();
  const { isLight } = useCoinWalletMode(coin);
  const visible = useWindowVisible();
  const queryClient = useQueryClient();

  useEffect(() => {
    if (!isLight || !visible) return;

    let cancelled = false;

    const tick = async () => {
      try {
        await lightWalletRefreshBalance(coin);
        if (cancelled) return;
        await queryClient.refetchQueries({
          queryKey: coinQueryKey(coin, 'getwalletinfo'),
          type: 'active',
        });
        await queryClient.invalidateQueries({
          queryKey: coinQueryKey(coin, 'wallet-cumulative-indexer'),
        });
        await queryClient.invalidateQueries({
          queryKey: coinQueryKey(coin, 'wallet-cumulative-txs'),
        });
      } catch {
        // Steady sync / foreground hooks still refresh in the background.
      }
    };

    void tick();
    const id = window.setInterval(() => void tick(), LIGHT_BALANCE_POLL_MS);
    return () => {
      cancelled = true;
      window.clearInterval(id);
    };
  }, [coin, isLight, queryClient, visible]);
}
