import { ArrowDownLeft, ArrowUpRight } from 'lucide-react';
import type { IndexerAddressEvent } from '@/lib/indexer-api';
import { formatSignedIndexerAmount, parseIndexerAmountCoins } from '@/lib/indexer-amount';
import { explorerAddressPath } from '@/lib/explorer-nav';
import { cn } from '@/lib/utils';
import { ExplorerInternalLink } from './ExplorerInternalLink';
import { shortExplorerAddress } from './tx-detail-utils';

function eventTone(ev: IndexerAddressEvent): 'receive' | 'send' | 'neutral' {
  const coins = parseIndexerAmountCoins(ev.delta);
  if (coins > 0) return 'receive';
  if (coins < 0) return 'send';
  return 'neutral';
}

export function WalletImpactCard({ events }: { events: IndexerAddressEvent[] }) {
  if (events.length === 0) return null;

  return (
    <section className="mobile-panel rounded-2xl border border-border bg-bg-panel/60 p-4">
      <h3 className="text-sm font-semibold text-fg">Balance changes</h3>
      <p className="mt-1 text-[11px] leading-relaxed text-fg-muted">
        How this transaction moved value for each address touched on-chain.
      </p>
      <ul className="mt-3 space-y-2">
        {events.map((ev) => {
          const tone = eventTone(ev);
          const label =
            ev.eventType === 'receive'
              ? 'Received'
              : ev.eventType === 'send'
                ? 'Sent'
                : tone === 'receive'
                  ? 'Received'
                  : tone === 'send'
                    ? 'Sent'
                    : 'Changed';

          return (
            <li
              key={`${ev.address}-${ev.eventType ?? tone}`}
              className={cn(
                'flex items-center gap-3 rounded-xl border px-3 py-2.5',
                tone === 'receive' && 'border-emerald-500/25 bg-emerald-500/8',
                tone === 'send' && 'border-amber-500/25 bg-amber-500/8',
                tone === 'neutral' && 'border-border/70 bg-bg-subtle/30'
              )}
            >
              <div
                className={cn(
                  'flex h-8 w-8 shrink-0 items-center justify-center rounded-lg',
                  tone === 'receive' && 'bg-emerald-500/15 text-emerald-700 dark:text-emerald-300',
                  tone === 'send' && 'bg-amber-500/15 text-amber-700 dark:text-amber-300',
                  tone === 'neutral' && 'bg-bg-subtle text-fg-muted'
                )}
              >
                {tone === 'receive' ? (
                  <ArrowDownLeft className="h-4 w-4" aria-hidden />
                ) : tone === 'send' ? (
                  <ArrowUpRight className="h-4 w-4" aria-hidden />
                ) : (
                  <ArrowUpRight className="h-4 w-4 opacity-50" aria-hidden />
                )}
              </div>
              <div className="min-w-0 flex-1">
                <p className="text-[11px] font-medium text-fg-muted">{label}</p>
                <ExplorerInternalLink to={explorerAddressPath(ev.address)} mono className="mt-0.5 block">
                  {shortExplorerAddress(ev.address)}
                </ExplorerInternalLink>
              </div>
              <p
                className={cn(
                  'shrink-0 text-sm font-semibold tabular-nums',
                  tone === 'receive' && 'text-emerald-700 dark:text-emerald-300',
                  tone === 'send' && 'text-amber-700 dark:text-amber-300',
                  tone === 'neutral' && 'text-fg'
                )}
              >
                {formatSignedIndexerAmount(ev.delta ?? undefined)}
              </p>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
