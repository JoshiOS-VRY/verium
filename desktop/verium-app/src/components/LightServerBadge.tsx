import { useQuery } from '@tanstack/react-query';
import { Cloud, CloudOff, Loader2 } from 'lucide-react';
import { useActiveCoin } from '@/lib/coin/context';
import { coinQueryKey } from '@/lib/coin/profile';
import { lightServerStatus } from '@/lib/light-wallet/client';
import {
  electrumServerShortLabel,
  electrumStatusTitle,
  formatChainHeight,
} from '@/lib/light-wallet/labels';
import { useWindowVisible } from '@/hooks/useWindowVisible';

export function LightServerBadge() {
  const activeCoin = useActiveCoin();
  const visible = useWindowVisible();
  const { data, isLoading, isError, error } = useQuery({
    queryKey: coinQueryKey(activeCoin, 'light-server-status'),
    queryFn: () => lightServerStatus(activeCoin),
    refetchInterval: visible ? 15_000 : false,
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
        title={isError ? String(error) : 'Could not reach the light wallet server'}
      >
        <CloudOff className="h-3.5 w-3.5" />
        Light wallet offline
      </span>
    );
  }

  const serverLabel = electrumServerShortLabel(activeCoin, data);
  const heightLabel =
    data.tip_height != null ? `block ${formatChainHeight(data.tip_height)}` : null;

  return (
    <span
      className="inline-flex items-center gap-1.5 text-xs text-emerald-600 dark:text-emerald-400"
      title={electrumStatusTitle(activeCoin, data)}
    >
      <Cloud className="h-3.5 w-3.5 shrink-0" />
      <span>
        {serverLabel}
        {heightLabel ? (
          <>
            <span className="text-fg-subtle"> · </span>
            {heightLabel}
          </>
        ) : null}
      </span>
    </span>
  );
}
