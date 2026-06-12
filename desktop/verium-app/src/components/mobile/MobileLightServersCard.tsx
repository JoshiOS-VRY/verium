import { useMutation, useQuery } from '@tanstack/react-query';
import { Loader2, Server } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { useActiveCoin } from '@/lib/coin/context';
import { electrumUriFriendlyLabel } from '@/lib/light-wallet/labels';
import { electrumServersGet, electrumTestConnection } from '@/lib/light-wallet/client';
import { MobileSettingsGroup } from '@/components/mobile/MobileSettingsGroup';

/** Mobile settings: Electrum servers only (no full-node mode UI). */
export function MobileLightServersCard() {
  const coin = useActiveCoin();
  const servers = useQuery({
    queryKey: ['electrum-servers', coin],
    queryFn: () => electrumServersGet(coin),
  });

  const testConn = useMutation({
    mutationFn: () => electrumTestConnection(coin),
  });

  return (
    <MobileSettingsGroup
      title="Light wallet servers"
      description="Vericonomy Electrum endpoints for balance and history."
      defaultOpen={false}
    >
      <ul className="flex flex-col gap-2 text-xs text-fg-muted">
        {(servers.data ?? []).map((uri, i) => (
          <li key={uri} className="rounded-lg border border-border/60 bg-bg-subtle/40 px-3 py-2">
            <span className="font-medium text-fg">{electrumUriFriendlyLabel(uri, coin, i)}</span>
            <span className="mt-0.5 block break-all font-mono text-[11px] text-fg-subtle">
              {uri}
            </span>
          </li>
        ))}
      </ul>
      <Button
        variant="secondary"
        className="mt-3 h-11 w-full rounded-xl"
        onClick={() => testConn.mutate()}
        disabled={testConn.isPending}
      >
        {testConn.isPending ? (
          <>
            <Loader2 className="h-4 w-4 animate-spin" />
            Testing…
          </>
        ) : (
          <>
            <Server className="h-4 w-4" />
            Test connection
          </>
        )}
      </Button>
      {testConn.data && (
        <p className={`mt-2 text-xs ${testConn.data.passed ? 'text-success' : 'text-danger'}`}>
          {testConn.data.server}: {testConn.data.passed ? 'Connected' : 'Failed'}
        </p>
      )}
      {testConn.error && <p className="mt-2 text-xs text-danger">{String(testConn.error)}</p>}
    </MobileSettingsGroup>
  );
}
