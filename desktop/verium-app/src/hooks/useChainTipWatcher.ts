import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";

import { coinQueryKey, type CoinId } from "@/lib/coin/profile";
import { pushChainTip } from "@/lib/chain-tip-store";
import { useWalletMode } from "@/hooks/useWalletMode";
import { walletTransactionsKeyPrefix } from "@/lib/wallet-transactions-query";

interface ChainTipPayload {
  coin: CoinId;
  height: number;
  hash: string;
  time: number;
}

/** Delay before refreshing the explorer feed so it can index the new block. */
const ENRICH_DELAY_MS = 4_000;
/** Coalesce burst tip events during sync (avoids invalidation storms). */
const INVALIDATE_DEBOUNCE_MS = 5_000;

/**
 * Listens for `chain-tip-changed` events from the node watcher, pushes them
 * into the shared chain tip store (instant UI updates), and triggers a
 * debounced explorer refetch to enrich the block with miner address/reward.
 */
export function useChainTipWatcher(): void {
  const { isLight } = useWalletMode();
  const queryClient = useQueryClient();

  useEffect(() => {
    if (isLight) return;
    let cancelled = false;
    let enrichTimer: number | undefined;
    let invalidateTimer: number | undefined;

    const scheduleInvalidations = (coin: CoinId) => {
      if (invalidateTimer != null) window.clearTimeout(invalidateTimer);
      invalidateTimer = window.setTimeout(() => {
        void queryClient.invalidateQueries({
          queryKey: coinQueryKey(coin, "getblockchaininfo"),
        });
        void queryClient.invalidateQueries({
          queryKey: coinQueryKey(coin, "getwalletinfo"),
        });
        // Wallet txs are heavy (listtransactions + header lookups) — refresh
        // on a slower cadence; block-found watchers also use chain-tip events.
        // Prefix-invalidate so both the shared 80-row poll and the Transactions
        // page "history" view refresh from this single event (neither needs an
        // aggressive standalone poll).
        void queryClient.invalidateQueries({
          queryKey: walletTransactionsKeyPrefix(coin),
        });
      }, INVALIDATE_DEBOUNCE_MS);
    };

    const unlistenPromise = listen<ChainTipPayload>("chain-tip-changed", (event) => {
      if (cancelled) return;
      const payload = event.payload;

      pushChainTip({
        coin: payload.coin,
        height: payload.height,
        hash: payload.hash,
        time: payload.time,
      });

      scheduleInvalidations(payload.coin);

      if (enrichTimer != null) window.clearTimeout(enrichTimer);
      enrichTimer = window.setTimeout(() => {
        void queryClient.invalidateQueries({
          queryKey: coinQueryKey(payload.coin, "explorer-blocks"),
        });
      }, ENRICH_DELAY_MS);
    });

    return () => {
      cancelled = true;
      if (enrichTimer != null) window.clearTimeout(enrichTimer);
      if (invalidateTimer != null) window.clearTimeout(invalidateTimer);
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, [isLight, queryClient]);
}
