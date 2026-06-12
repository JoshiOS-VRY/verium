import type { ReactNode } from 'react';
import { ArrowDownLeft, ArrowUpRight, Calendar, Hash, Layers, Trophy } from 'lucide-react';
import { Badge } from '@/components/ui/Badge';
import { ExplorerInternalLink } from '@/components/mobile/explorer/ExplorerInternalLink';
import type { IndexerAddressBalance, IndexerAddressRichlist } from '@/lib/indexer-api';
import { formatIndexerAmount } from '@/lib/indexer-api';
import { explorerBlockPath } from '@/lib/explorer-nav';
import { formatTransactionTime } from '@/lib/units';

function StatCard({
  icon,
  label,
  value,
  sub,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
  sub?: ReactNode;
}) {
  return (
    <div className="min-w-[9.5rem] shrink-0 rounded-xl border border-border/70 bg-bg-subtle/30 px-3 py-2.5">
      <div className="flex items-center gap-1.5 text-fg-muted">
        {icon}
        <span className="text-[10px] font-medium uppercase tracking-wide">{label}</span>
      </div>
      <p className="mt-1 text-sm font-semibold tabular-nums text-fg">{value}</p>
      {sub ? <p className="mt-0.5 text-[10px] text-fg-subtle">{sub}</p> : null}
    </div>
  );
}

export function AddressStatsStrip({
  balance,
  richlist,
}: {
  balance?: IndexerAddressBalance | null;
  richlist?: IndexerAddressRichlist | null;
}) {
  if (!balance) return null;

  const txCount = balance.txCount != null ? balance.txCount.toLocaleString() : '—';

  return (
    <section className="space-y-2">
      {richlist?.rank != null && (
        <div className="flex items-center gap-2 px-1">
          <Badge tone="accent" className="gap-1">
            <Trophy className="h-3 w-3" aria-hidden />
            Rich list #{richlist.rank.toLocaleString()}
            {richlist.percentile != null ? ` · top ${(100 - richlist.percentile).toFixed(1)}%` : ''}
          </Badge>
        </div>
      )}

      <div className="-mx-1 flex gap-2 overflow-x-auto px-1 pb-1 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
        <StatCard
          icon={<ArrowDownLeft className="h-3.5 w-3.5 text-emerald-600 dark:text-emerald-400" />}
          label="Received"
          value={formatIndexerAmount(balance.totalReceived ?? undefined)}
        />
        <StatCard
          icon={<ArrowUpRight className="h-3.5 w-3.5 text-amber-600 dark:text-amber-400" />}
          label="Sent"
          value={formatIndexerAmount(balance.totalSent ?? undefined)}
        />
        <StatCard icon={<Hash className="h-3.5 w-3.5" />} label="Transactions" value={txCount} />
        {balance.firstSeenTime != null && (
          <StatCard
            icon={<Calendar className="h-3.5 w-3.5" />}
            label="First seen"
            value={formatTransactionTime(balance.firstSeenTime)}
            sub={
              balance.firstSeenHeight != null
                ? `Block ${balance.firstSeenHeight.toLocaleString()}`
                : undefined
            }
          />
        )}
        {balance.lastSeenHeight != null && (
          <StatCard
            icon={<Layers className="h-3.5 w-3.5" />}
            label="Last active"
            value={`#${balance.lastSeenHeight.toLocaleString()}`}
            sub={
              <ExplorerInternalLink
                to={explorerBlockPath(balance.lastSeenHeight)}
                className="text-[10px]"
              >
                View block
              </ExplorerInternalLink>
            }
          />
        )}
      </div>
    </section>
  );
}
