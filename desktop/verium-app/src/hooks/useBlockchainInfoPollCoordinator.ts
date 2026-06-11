import { useQuery } from '@tanstack/react-query';
import { useActiveCoin } from '@/lib/coin/context';
import { coinQueryKey, type CoinId } from '@/lib/coin/profile';
import { useUserPreferences } from '@/lib/user-preferences';
import { useCoinWalletMode } from '@/hooks/useWalletMode';
import { useWindowVisible } from '@/hooks/useWindowVisible';
import { rpcGetBlockchainInfo, type BlockchainInfo } from '@/lib/rpc/client';

const CHAIN_POLL_MS = 30_000;
const CHAIN_SYNC_POLL_MS = 15_000;

function pollInterval(data: BlockchainInfo | undefined, visible: boolean) {
  if (!visible) return false;
  const syncing =
    data != null && data.headers != null && data.blocks != null && data.headers > data.blocks + 1;
  return syncing || data?.initialblockdownload ? CHAIN_SYNC_POLL_MS : CHAIN_POLL_MS;
}

function inactiveCoinNeedsBackgroundPoll(
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
 * Single writer for `getblockchaininfo` — other hooks must use `refetchInterval: false`.
 */
export function useBlockchainInfoPollCoordinator(): void {
  const activeCoin = useActiveCoin();
  const visible = useWindowVisible();
  const veriumMode = useCoinWalletMode('verium');
  const vericoinMode = useCoinWalletMode('vericoin');
  const prefs = useUserPreferences((s) => s.prefs);

  const pollVerium =
    !veriumMode.isLight &&
    prefs.verium_enabled !== false &&
    (activeCoin === 'verium' || inactiveCoinNeedsBackgroundPoll('verium', prefs));
  const pollVericoin =
    !vericoinMode.isLight &&
    prefs.vericoin_enabled !== false &&
    (activeCoin === 'vericoin' || inactiveCoinNeedsBackgroundPoll('vericoin', prefs));

  useQuery({
    queryKey: coinQueryKey('verium', 'getblockchaininfo'),
    queryFn: () => rpcGetBlockchainInfo('verium'),
    enabled: pollVerium,
    refetchInterval: (q) => pollInterval(q.state.data ?? undefined, visible),
    staleTime: 10_000,
    gcTime: 30_000,
  });

  useQuery({
    queryKey: coinQueryKey('vericoin', 'getblockchaininfo'),
    queryFn: () => rpcGetBlockchainInfo('vericoin'),
    enabled: pollVericoin,
    refetchInterval: (q) => pollInterval(q.state.data ?? undefined, visible),
    staleTime: 10_000,
    gcTime: 30_000,
  });
}
