import { useQuery } from '@tanstack/react-query';
import { Cloud, CloudOff, Loader2 } from 'lucide-react';
import { useActiveCoin } from '@/lib/coin/context';
import { coinQueryKey } from '@/lib/coin/profile';
import { lightServerStatus } from '@/lib/light-wallet/client';
import { LIGHT_SERVER_STATUS_POLL_MS } from '@/lib/light-wallet/poll';
import { electrumStatusTitle } from '@/lib/light-wallet/labels';
import { useWindowVisible } from '@/hooks/useWindowVisible';

export function LightServerBadge() {
  const activeCoin = useActiveCoin();
  const visible = useWindowVisible();
  const { data, isLoading, isError, error } = useQuery({
    queryKey: coinQueryKey(activeCoin, 'light-server-status'),
    queryFn: () => lightServerStatus(activeCoin),
    refetchInterval: visible ? LIGHT_SERVER_STATUS_POLL_MS : false,
    retry: 1,
  });

  if (isLoading) {
    return (
      <span className="inline-flex items-center gap-1.5 text-xs text-muted-foreground">
        <Loader2 className="h-3.5 w-3.5 animate-spin" />
        Connecting to light wallet…
      </span>
    );
  }

  if (data == null && !isLoading && !isError) {
    return null;
  }

  if (isError || !data?.connected) {
    return (
      <span
        className="inline-flex items-center gap-1.5 text-xs text-amber-600 dark:text-amber-400"
        title={
          isError
            ? String(error)
            : 'Could not reach the Vericonomy Electrum server (balances may use cache)'
        }
      >
        <CloudOff className="h-3.5 w-3.5" />
        Sync server offline
      </span>
    );
  }

  return (
    <span
      className="inline-flex min-w-0 max-w-full items-center gap-1.5 text-xs text-emerald-600 dark:text-emerald-400"
      title={electrumStatusTitle(activeCoin, data)}
    >
      <span className="relative flex h-3.5 w-3.5 shrink-0" aria-hidden>
        <span className="absolute inline-flex h-full w-full rounded-full" />
        <Cloud className="relative h-3.5 w-3.5" />
      </span>
      <span className="min-w-0 truncate">Connected</span>
    </span>
  );
}
