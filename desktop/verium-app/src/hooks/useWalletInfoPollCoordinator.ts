import { useQuery } from '@tanstack/react-query';
import { useActiveCoin } from '@/lib/coin/context';
import { coinQueryKey, type CoinId } from '@/lib/coin/profile';
import { useWindowVisible } from '@/hooks/useWindowVisible';
import { useUserPreferences } from '@/lib/user-preferences';
import { useCoinWalletMode } from '@/hooks/useWalletMode';
import { lightWalletExists } from '@/lib/light-wallet/client';
import { rpcGetWalletInfo, type WalletInfo } from '@/lib/rpc/client';

function walletScanProgress(
  scanning: WalletInfo['scanning']
): { duration: number; progress: number } | null {
  return typeof scanning === 'object' ? scanning : null;
}

const WALLET_INFO_POLL_MS = 30_000;
const WALLET_SCAN_POLL_MS = 5_000;

function inactiveCoinNeedsBackgroundPoll(
  coin: CoinId,
  prefs: {
    verium_enabled?: boolean;
    vericoin_enabled?: boolean;
    auto_mine_on_open?: boolean;
    auto_stake_on_open?: boolean;
    notify_on_vrm_received?: boolean;
    notify_on_vrc_received?: boolean;
    play_sound_on_stake_reward?: boolean;
  }
): boolean {
  if (coin === 'verium') {
    return (
      prefs.verium_enabled !== false &&
      (prefs.auto_mine_on_open === true || prefs.notify_on_vrm_received !== false)
    );
  }
  return (
    prefs.vericoin_enabled !== false &&
    (prefs.auto_stake_on_open === true ||
      prefs.notify_on_vrc_received !== false ||
      prefs.play_sound_on_stake_reward === true)
  );
}

/**
 * Single writer for `getwalletinfo` — other hooks must use `refetchInterval: false`.
 * Polls the active coin always; the inactive coin only when a background feature
 * (auto-mine/stake, notifications) needs it so coin switches do not keep both
 * full-node daemons hot.
 */
export function useWalletInfoPollCoordinator(): void {
  const activeCoin = useActiveCoin();
  const visible = useWindowVisible();
  const prefs = useUserPreferences((s) => s.prefs);
  const veriumMode = useCoinWalletMode('verium');
  const vericoinMode = useCoinWalletMode('vericoin');
  const veriumLight = useQuery({
    queryKey: coinQueryKey('verium', 'light-wallet-exists'),
    queryFn: () => lightWalletExists('verium'),
    enabled: prefs.verium_enabled !== false,
    staleTime: 30_000,
  });
  const vericoinLight = useQuery({
    queryKey: coinQueryKey('vericoin', 'light-wallet-exists'),
    queryFn: () => lightWalletExists('vericoin'),
    enabled: prefs.vericoin_enabled !== false,
    staleTime: 30_000,
  });

  const pollVerium =
    prefs.verium_enabled !== false &&
    (activeCoin === 'verium' || inactiveCoinNeedsBackgroundPoll('verium', prefs));
  const pollVericoin =
    prefs.vericoin_enabled !== false &&
    (activeCoin === 'vericoin' || inactiveCoinNeedsBackgroundPoll('vericoin', prefs));

  const interval = (data: WalletInfo | undefined) => {
    if (!visible) return false;
    if (data?.light_syncing) return WALLET_SCAN_POLL_MS;
    return walletScanProgress(data?.scanning) ? WALLET_SCAN_POLL_MS : WALLET_INFO_POLL_MS;
  };

  useQuery({
    queryKey: coinQueryKey('verium', 'getwalletinfo'),
    queryFn: () => rpcGetWalletInfo('verium'),
    enabled: pollVerium && (!veriumMode.isLight || veriumLight.data !== false),
    refetchInterval: (q) => interval(q.state.data ?? undefined),
    staleTime: 10_000,
    gcTime: 30_000,
  });

  useQuery({
    queryKey: coinQueryKey('vericoin', 'getwalletinfo'),
    queryFn: () => rpcGetWalletInfo('vericoin'),
    enabled: pollVericoin && (!vericoinMode.isLight || vericoinLight.data !== false),
    refetchInterval: (q) => interval(q.state.data ?? undefined),
    staleTime: 10_000,
    gcTime: 30_000,
  });
}
