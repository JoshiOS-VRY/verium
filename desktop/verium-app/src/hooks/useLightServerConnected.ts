import { useQuery } from '@tanstack/react-query';
import { useActiveCoin } from '@/lib/coin/context';
import { coinQueryKey } from '@/lib/coin/profile';
import { lightServerStatus } from '@/lib/light-wallet/client';
import { useWalletMode } from '@/hooks/useWalletMode';
import { useWindowVisible } from '@/hooks/useWindowVisible';

/** Electrum reachability for light mode (replaces daemon `connected` in UI gates). */
export function useLightServerConnected() {
  const coin = useActiveCoin();
  const { isLight } = useWalletMode();
  const visible = useWindowVisible();

  const query = useQuery({
    queryKey: coinQueryKey(coin, 'light-server-status'),
    queryFn: () => lightServerStatus(coin),
    enabled: isLight,
    refetchInterval: isLight && visible ? 45_000 : false,
    staleTime: 10_000,
  });

  const connected = isLight && query.data?.connected === true;

  return {
    ...query,
    connected,
    tipHeight: query.data?.tip_height ?? null,
    isLight,
  };
}
