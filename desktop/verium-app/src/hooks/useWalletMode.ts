import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useActiveCoin } from '@/lib/coin/context';
import type { CoinId } from '@/lib/coin/profile';
import { LIGHT_WALLET_ENABLED } from '@/lib/features';
import { walletModeGetForCoin, type WalletModeStatus } from '@/lib/light-wallet/client';

/** Prefix for invalidating wallet-mode queries (all coins). */
export const WALLET_MODE_QUERY_KEY = ['wallet-mode-status'] as const;

export function walletModeQueryKey(coin: CoinId) {
  return [...WALLET_MODE_QUERY_KEY, coin] as const;
}

function modeFromStatus(data: WalletModeStatus | undefined) {
  const mobileOnly = data?.mobile_only === true;
  const lightWalletEnabled =
    mobileOnly || LIGHT_WALLET_ENABLED || data?.light_wallet_enabled === true;
  const mode = mobileOnly ? 'light' : (data?.mode ?? 'full_node');
  const isLight = mobileOnly || (lightWalletEnabled && data?.mode === 'light');
  return {
    isLight,
    isFullNode: !isLight,
    mode,
    lightWalletEnabled,
    lightWalletExists: data?.light_wallet_exists ?? false,
    electrumServers: data?.electrum_servers ?? [],
    mobileOnly,
  };
}

/** Effective wallet mode for one chain (use in coordinators that poll a fixed coin). */
export function useCoinWalletMode(coin: CoinId) {
  const { data, isLoading } = useQuery({
    queryKey: walletModeQueryKey(coin),
    queryFn: () => walletModeGetForCoin(coin),
    staleTime: 30_000,
  });
  return { isLoading, ...modeFromStatus(data) };
}

/** Wallet mode for the active chain (UI gates, banners, unlock forms). */
export function useWalletMode() {
  const coin = useActiveCoin();
  const { isLoading, ...mode } = useCoinWalletMode(coin);
  return { isLoading, ...mode };
}

export function useInvalidateWalletMode() {
  const queryClient = useQueryClient();
  return () => {
    void queryClient.invalidateQueries({ queryKey: WALLET_MODE_QUERY_KEY });
  };
}
