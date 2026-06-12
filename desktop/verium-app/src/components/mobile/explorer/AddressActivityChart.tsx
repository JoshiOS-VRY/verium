import { Activity } from 'lucide-react';
import {
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts';
import type { AddressActivityBar } from '@/lib/address-activity';
import { formatIndexerCoinsTotal, indexerTicker } from '@/lib/indexer-amount';
import type { IndexerAmount } from '@/lib/indexer-api';

function ChartTooltip({
  active,
  payload,
  ticker,
}: {
  active?: boolean;
  payload?: { payload: AddressActivityBar }[];
  ticker: string;
}) {
  if (!active || !payload?.[0]?.payload) return null;
  const row = payload[0].payload;
  const type = row.isCoinbase ? 'Coinbase' : row.isCoinstake ? 'Coinstake' : 'Transfer';

  return (
    <div className="rounded-lg border border-border bg-bg-panel px-3 py-2 text-xs shadow-md">
      <p className="font-medium text-fg">{row.label}</p>
      <p className="mt-0.5 tabular-nums text-fg-muted">
        {formatIndexerCoinsTotal(row.delta, ticker)} · {type}
      </p>
    </div>
  );
}

export function AddressActivityChart({
  bars,
  sampleBalance,
}: {
  bars: AddressActivityBar[];
  sampleBalance?: IndexerAmount | null;
}) {
  const ticker = indexerTicker(sampleBalance) || 'VRM';

  if (bars.length === 0) {
    return (
      <section className="mobile-panel rounded-2xl border border-border bg-bg-panel/60 p-4">
        <h3 className="text-sm font-semibold text-fg">Recent activity</h3>
        <p className="mt-2 text-xs text-fg-muted">No chartable transactions in this sample.</p>
      </section>
    );
  }

  return (
    <section className="mobile-panel rounded-2xl border border-border bg-bg-panel/60 p-4">
      <div className="flex items-center gap-2">
        <Activity className="h-4 w-4 text-accent" aria-hidden />
        <h3 className="text-sm font-semibold text-fg">Recent activity</h3>
      </div>
      <p className="mt-1 text-[11px] text-fg-muted">
        Net change per transaction (latest {bars.length} indexed legs).
      </p>

      <div className="mt-3 h-44 w-full min-w-0">
        <ResponsiveContainer width="100%" height="100%">
          <BarChart data={bars} margin={{ top: 4, right: 4, left: -18, bottom: 0 }}>
            <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" vertical={false} />
            <XAxis
              dataKey="label"
              tick={{ fontSize: 10, fill: 'var(--fg-muted)' }}
              interval="preserveStartEnd"
              minTickGap={16}
            />
            <YAxis
              tick={{ fontSize: 10, fill: 'var(--fg-muted)' }}
              width={42}
              tickFormatter={(v) => (Math.abs(v) >= 1 ? v.toFixed(1) : v.toFixed(3))}
            />
            <Tooltip
              content={<ChartTooltip ticker={ticker} />}
              cursor={{ fill: 'var(--bg-subtle)' }}
            />
            <Bar dataKey="delta" radius={[3, 3, 0, 0]} maxBarSize={28}>
              {bars.map((entry) => (
                <Cell
                  key={entry.id}
                  fill={entry.delta >= 0 ? 'var(--success)' : 'var(--warning)'}
                />
              ))}
            </Bar>
          </BarChart>
        </ResponsiveContainer>
      </div>

      <div className="mt-2 flex justify-between text-[10px] text-fg-subtle">
        <span className="inline-flex items-center gap-1">
          <span className="h-2 w-2 rounded-sm bg-success/75" />
          Incoming
        </span>
        <span className="inline-flex items-center gap-1">
          <span className="h-2 w-2 rounded-sm bg-warning/75" />
          Outgoing
        </span>
      </div>
    </section>
  );
}
