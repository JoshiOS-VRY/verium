import { useQuery } from '@tanstack/react-query';
import { useActiveCoin } from '@/lib/coin/context';
import { coinQueryKey, type CoinId } from '@/lib/coin/profile';
import { useExplorerQueriesEnabled } from '@/lib/network-mode';
import { useUserPreferences } from '@/lib/user-preferences';
import { useCoinWalletMode } from '@/hooks/useWalletMode';
import { useWindowVisible } from '@/hooks/useWindowVisible';
import { fetchExplorerStats } from '@/lib/explorer-api';

/** Single writer interval for explorer network tip / stats. */
export const EXPLORER_STATS_POLL_MS = 60_000;

function inactiveCoinNeedsExplorerPoll(
  coin: CoinId,
  prefs: {
    verium_enabled?: boolean;
    vericoin_enabled?: boolean;
    auto_mine_on_open?: boolean;
    auto_stake_on_open?: boolean;
  }
): boolean {
  if (coin === 'verium') {
    return prefs.verium_enabled !== false && prefs.auto_mine_on_open === true;
  }
  return prefs.vericoin_enabled !== false && prefs.auto_stake_on_open === true;
}

/**
 * Single writer for `explorer-stats` — other hooks must use `refetchInterval: false`.
 *
 * Without this, ~10 dashboard/route observers each set 30–60s intervals and React
 * Query takes the minimum, multiplying HTTP + deserialization load in WebView2.
 */
export function useExplorerStatsPollCoordinator(): void {
  const activeCoin = useActiveCoin();
  const visible = useWindowVisible();
  const explorerEnabled = useExplorerQueriesEnabled();
  const veriumMode = useCoinWalletMode('verium');
  const vericoinMode = useCoinWalletMode('vericoin');
  const prefs = useUserPreferences((s) => s.prefs);

  const pollVerium =
    explorerEnabled &&
    !veriumMode.isLight &&
    prefs.verium_enabled !== false &&
    (activeCoin === 'verium' || inactiveCoinNeedsExplorerPoll('verium', prefs));
  const pollVericoin =
    explorerEnabled &&
    !vericoinMode.isLight &&
    prefs.vericoin_enabled !== false &&
    (activeCoin === 'vericoin' || inactiveCoinNeedsExplorerPoll('vericoin', prefs));

  useQuery({
    queryKey: coinQueryKey('verium', 'explorer-stats'),
    queryFn: () => fetchExplorerStats('verium'),
    enabled: pollVerium,
    refetchInterval: visible ? EXPLORER_STATS_POLL_MS : false,
    staleTime: 30_000,
    gcTime: 30_000,
    retry: 0,
  });

  useQuery({
    queryKey: coinQueryKey('vericoin', 'explorer-stats'),
    queryFn: () => fetchExplorerStats('vericoin'),
    enabled: pollVericoin,
    refetchInterval: visible ? EXPLORER_STATS_POLL_MS : false,
    staleTime: 30_000,
    gcTime: 30_000,
    retry: 0,
  });
}
