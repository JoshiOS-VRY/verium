import { useEffect, useRef } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { useEnabledCoins } from '@/lib/coin/context';
import type { CoinId } from '@/lib/coin/profile';
import { lightWalletRefreshPending, lightServerStatus } from '@/lib/light-wallet/client';
import {
  LIGHT_INCOMING_NOTIFY_BACKGROUND_POLL_MS,
  LIGHT_INCOMING_NOTIFY_POLL_MS,
} from '@/lib/light-wallet/poll';
import { walletTransactionsKeyPrefix } from '@/lib/wallet-transactions-query';
import { useCoinWalletMode } from '@/hooks/useWalletMode';
import { useWindowVisible } from '@/hooks/useWindowVisible';

/**
 * Light wallets: poll Electrum tip and merge pending / newly confirmed txs into the
 * local history cache so activity and confirmations are not stuck on explorer lag.
 */
export function useLightIncomingChainSync(): void {
  const enabledCoins = useEnabledCoins();
  const veriumMode = useCoinWalletMode('verium');
  const vericoinMode = useCoinWalletMode('vericoin');
  const visible = useWindowVisible();
  const queryClient = useQueryClient();
  const lastTip = useRef<Record<string, number>>({});

  const coins: CoinId[] = enabledCoins.filter((coin) => {
    if (coin === 'verium') return veriumMode.isLight;
    if (coin === 'vericoin') return vericoinMode.isLight;
    return false;
  });

  useEffect(() => {
    if (coins.length === 0) return;

    let cancelled = false;
    const intervalMs = visible
      ? LIGHT_INCOMING_NOTIFY_POLL_MS
      : LIGHT_INCOMING_NOTIFY_BACKGROUND_POLL_MS;

    const refreshCoin = async (coin: CoinId, tipAdvanced: boolean) => {
      try {
        await lightWalletRefreshPending(coin);
      } catch {
        // ignore — locked or offline
      }
      if (tipAdvanced || !cancelled) {
        await queryClient.invalidateQueries({ queryKey: walletTransactionsKeyPrefix(coin) });
      }
    };

    const tick = async () => {
      for (const coin of coins) {
        if (cancelled) return;
        try {
          const status = await lightServerStatus(coin);
          const tip = status?.tip_height ?? null;
          const prev = lastTip.current[coin] ?? 0;
          const advanced = tip != null && tip > prev;
          if (tip != null) {
            lastTip.current[coin] = tip;
          }
          await refreshCoin(coin, advanced);
        } catch {
          await refreshCoin(coin, false);
        }
      }
    };

    void tick();
    const id = window.setInterval(() => void tick(), intervalMs);
    return () => {
      cancelled = true;
      window.clearInterval(id);
    };
  }, [coins, queryClient, visible]);
}
