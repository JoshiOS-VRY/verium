import { TrendingUp } from 'lucide-react';
import { AnimatedBlockNumber } from '@/components/AnimatedBlockNumber';
import { BlockAgeLabel } from '@/components/BlockAgeLabel';
import { ExplorerLink } from '@/components/ExplorerLink';
import { useDashboardData } from '@/hooks/useDashboardData';
import { getCoinProfile, type CoinId } from '@/lib/coin/profile';
import { cn, formatNumber } from '@/lib/utils';

function formatUsd(value?: number | null): string {
  if (value === undefined || value === null) return '—';
  if (value >= 1_000_000) return `$${formatNumber(value / 1_000_000, 2)}M`;
  if (value >= 1_000) return `$${formatNumber(value / 1_000, 2)}K`;
  return `$${formatNumber(value, 4)}`;
}

export function MobileDashboardHero({ coin }: { coin: CoinId }) {
  const profile = getCoinProfile(coin);
  const data = useDashboardData(coin);
  const priceUsd = data.explorer.data?.price_usd;
  const online = data.connected;

  return (
    <section className="mobile-panel flex items-center gap-3 rounded-2xl border border-border bg-bg-panel px-3 py-2.5 shadow-sm">
      <span
        className={cn(
          'inline-flex shrink-0 items-center gap-1 rounded-full px-2 py-0.5 text-[10px] font-semibold',
          online ? 'bg-success/10 text-success' : 'bg-warning/10 text-warning'
        )}
      >
        <span className={cn('h-1.5 w-1.5 rounded-full', online ? 'bg-success' : 'bg-warning')} />
        {online ? 'Connected' : 'Offline'}
      </span>

      <div className="min-w-0 flex-1">
        <div className="flex flex-wrap items-baseline gap-x-2 gap-y-0.5">
          <span className="text-[10px] font-medium uppercase tracking-wide text-fg-subtle">
            Chain tip
          </span>
          <AnimatedBlockNumber
            value={data.tipHeight}
            className="text-base font-bold tabular-nums text-fg"
            fallback={data.activity.showSpinner ? '…' : '—'}
          />
          <span className="text-[10px] text-fg-muted [&_p]:inline [&_p]:mt-0">
            <BlockAgeLabel tipTime={data.tipTime} />
          </span>
        </div>
      </div>

      {priceUsd != null && (
        <div className="hidden shrink-0 items-center gap-1 sm:flex">
          <TrendingUp className="h-3.5 w-3.5 text-accent" aria-hidden />
          <span className="text-xs font-semibold tabular-nums text-fg">{formatUsd(priceUsd)}</span>
          <span className="text-[10px] text-fg-subtle">{profile.symbol}</span>
        </div>
      )}

      {data.tipHash && (
        <ExplorerLink
          coin={coin}
          target={{ kind: 'block', hashOrHeight: data.tipHash }}
          label="View"
          className="shrink-0 text-xs"
        />
      )}
    </section>
  );
}
