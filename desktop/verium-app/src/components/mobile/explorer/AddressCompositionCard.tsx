import { PieChart } from 'lucide-react';
import type { AddressComposition } from '@/lib/address-activity';
import { formatIndexerCoinsTotal } from '@/lib/indexer-amount';
import { cn } from '@/lib/utils';

function pct(part: number, total: number): number {
  if (total <= 0) return 0;
  return Math.min(100, Math.max(0, (part / total) * 100));
}

export function AddressCompositionCard({ composition }: { composition: AddressComposition }) {
  const { received, sent, balance, ticker } = composition;
  const sentPct = pct(sent, received);
  const balancePct = pct(balance, received);
  const outflowPct = Math.min(100, sentPct + balancePct);

  return (
    <section className="mobile-panel rounded-2xl border border-border bg-bg-panel/60 p-4">
      <div className="flex items-center gap-2">
        <PieChart className="h-4 w-4 text-accent" aria-hidden />
        <h3 className="text-sm font-semibold text-fg">Balance breakdown</h3>
      </div>
      <p className="mt-1 text-[11px] text-fg-muted">
        How lifetime received coins split between current balance and amounts sent away.
      </p>

      <div className="mt-4 space-y-3">
        <div>
          <div className="flex justify-between text-[11px]">
            <span className="text-fg-muted">Lifetime received</span>
            <span className="font-medium tabular-nums text-fg">
              {formatIndexerCoinsTotal(received, ticker)}
            </span>
          </div>
          <div className="mt-1.5 h-3 overflow-hidden rounded-full bg-bg-subtle">
            <div className="h-full w-full rounded-full bg-emerald-500/35" />
          </div>
        </div>

        <div>
          <div className="flex justify-between text-[11px]">
            <span className="text-fg-muted">Still on address</span>
            <span className="font-medium tabular-nums text-emerald-700 dark:text-emerald-300">
              {formatIndexerCoinsTotal(balance, ticker)}
            </span>
          </div>
          <div className="mt-1.5 h-3 overflow-hidden rounded-full bg-bg-subtle">
            <div
              className="h-full rounded-full bg-emerald-500/70 transition-all"
              style={{ width: `${balancePct}%` }}
            />
          </div>
        </div>

        <div>
          <div className="flex justify-between text-[11px]">
            <span className="text-fg-muted">Sent out</span>
            <span className="font-medium tabular-nums text-amber-700 dark:text-amber-300">
              {formatIndexerCoinsTotal(sent, ticker)}
            </span>
          </div>
          <div className="mt-1.5 h-3 overflow-hidden rounded-full bg-bg-subtle">
            <div
              className="h-full rounded-full bg-amber-500/70 transition-all"
              style={{ width: `${sentPct}%` }}
            />
          </div>
        </div>
      </div>

      <div className="mt-4 flex h-2 overflow-hidden rounded-full bg-bg-subtle">
        <div
          className="bg-emerald-500/80"
          style={{ width: `${balancePct}%` }}
          title="Balance"
        />
        <div
          className="bg-amber-500/80"
          style={{ width: `${sentPct}%` }}
          title="Sent"
        />
        {outflowPct < 100 && (
          <div className="flex-1 bg-transparent" title="Unallocated indexing gap" />
        )}
      </div>
      <p className="mt-2 flex flex-wrap gap-x-3 gap-y-1 text-[10px] text-fg-subtle">
        <span className="inline-flex items-center gap-1">
          <span className={cn('h-2 w-2 rounded-full bg-emerald-500/80')} />
          Balance
        </span>
        <span className="inline-flex items-center gap-1">
          <span className={cn('h-2 w-2 rounded-full bg-amber-500/80')} />
          Sent
        </span>
      </p>
    </section>
  );
}
