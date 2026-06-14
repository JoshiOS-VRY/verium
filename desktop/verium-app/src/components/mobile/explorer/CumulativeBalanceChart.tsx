import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { LineChart as LineChartIcon } from 'lucide-react';
import {
  Area,
  AreaChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts';
import type { MouseHandlerDataParam } from 'recharts/types/synchronisation/types';
import type { CoinId } from '@/lib/coin/profile';
import { pointFromChartState, scrubPointFromClientX } from '@/lib/chart-scrub';
import {
  computeChartBalanceDomain,
  formatChartAxisBalance,
  formatChartPointDate,
  type ChartCumulativePoint,
} from '@/lib/cumulative-balance';
import { formatCoinAmount } from '@/lib/units';
import { cn } from '@/lib/utils';

const CHART_MARGIN = { top: 8, right: 12, left: 0, bottom: 0 };

function formatPinnedBalance(balance: number, coin: CoinId | undefined, ticker: string): string {
  if (coin != null) return formatCoinAmount(balance, coin, 4);
  return `${formatChartAxisBalance(balance)} ${ticker}`;
}

function estimateYAxisWidth(domain: [number, number]): number {
  const hiLabel = formatChartAxisBalance(domain[1]);
  const loLabel = formatChartAxisBalance(domain[0]);
  const chars = Math.max(hiLabel.length, loLabel.length);
  return Math.min(56, Math.max(36, chars * 6 + 10));
}

function PinnedChartValue({
  point,
  scrubbing,
  coin,
  ticker,
}: {
  point: ChartCumulativePoint;
  scrubbing: boolean;
  coin?: CoinId;
  ticker: string;
}) {
  return (
    <div
      className="rounded-lg border border-border/60 bg-bg-subtle/70 px-3 py-2.5"
      aria-live="polite"
    >
      <p className="text-[10px] font-medium uppercase tracking-wide text-fg-subtle">
        {scrubbing ? 'Selected balance' : 'Current balance'}
      </p>
      <p className="mt-0.5 text-xl font-bold tabular-nums tracking-tight text-fg">
        {formatPinnedBalance(point.balance, coin, ticker)}
      </p>
      <p className="mt-0.5 text-xs text-fg-muted">{formatChartPointDate(point.time, true)}</p>
    </div>
  );
}

export function CumulativeBalanceChartSkeleton({
  embedded = false,
  className,
}: {
  embedded?: boolean;
  className?: string;
}) {
  const body = (
    <div className={cn('animate-pulse', embedded ? 'mt-3' : 'mt-3')}>
      {!embedded && (
        <div className="rounded-lg border border-border/60 bg-bg-subtle/70 px-3 py-2.5">
          <div className="h-2.5 w-24 rounded bg-bg-subtle" />
          <div className="mt-2 h-7 w-32 rounded bg-bg-subtle" />
          <div className="mt-2 h-3 w-28 rounded bg-bg-subtle" />
        </div>
      )}
      <div
        className={cn(
          'relative overflow-hidden rounded-lg bg-bg-subtle/50',
          embedded ? 'mt-1 h-36' : 'mt-2 h-52'
        )}
        aria-hidden
      >
        <div className="absolute inset-x-0 bottom-8 h-px bg-border/80" />
        <div className="absolute bottom-8 left-[12%] right-[8%] top-[35%] rounded-t-lg bg-accent/10" />
        <div className="absolute bottom-8 left-[12%] h-[45%] w-[55%] rounded-tl-lg bg-accent/15" />
      </div>
    </div>
  );

  if (embedded) {
    return (
      <div className={cn('border-t border-border/60 pt-3', className)} aria-busy="true">
        {body}
      </div>
    );
  }

  return (
    <section
      className={cn('mobile-panel rounded-2xl border border-border bg-bg-panel/60 p-4', className)}
      aria-busy="true"
    >
      <div className="flex items-center gap-2">
        <LineChartIcon className="h-4 w-4 text-accent/40" aria-hidden />
        <div className="h-4 w-36 animate-pulse rounded bg-bg-subtle" />
      </div>
      {body}
    </section>
  );
}

export function CumulativeBalanceChart({
  points,
  ticker,
  caption,
  title = 'Cumulative balance',
  className,
  blurClass,
  anchorBalance,
  coin,
  embedded = false,
  onScrubChange,
}: {
  points: ChartCumulativePoint[];
  ticker: string;
  caption?: string;
  title?: string;
  className?: string;
  blurClass?: string;
  /** Ensures axis domain matches the live wallet/address balance. */
  anchorBalance?: number;
  coin?: CoinId;
  embedded?: boolean;
  /** Fired when the user scrubs the chart (embedded hero balance). */
  onScrubChange?: (point: ChartCumulativePoint | null, scrubbing: boolean) => void;
}) {
  const defaultPoint = points[points.length - 1];
  const [scrubbedPoint, setScrubbedPoint] = useState<ChartCumulativePoint | null>(null);
  const [scrubbing, setScrubbing] = useState(false);
  const chartAreaRef = useRef<HTMLDivElement>(null);
  const scrubbingRef = useRef(false);

  const domain = useMemo(
    () => computeChartBalanceDomain(points, anchorBalance),
    [points, anchorBalance]
  );
  const yAxisWidth = useMemo(() => estimateYAxisWidth(domain), [domain]);
  const xDomain = useMemo((): [number, number] => {
    if (points.length === 0) return [0, 1];
    const times = points.map((p) => p.time);
    const min = Math.min(...times);
    const max = Math.max(...times);
    const span = max - min;
    const pad = span > 0 ? span * 0.02 : 3600;
    return [min - pad, max + pad];
  }, [points]);

  const displayPoint = scrubbedPoint ?? defaultPoint;

  useEffect(() => {
    if (!onScrubChange) return;
    if (scrubbing) {
      onScrubChange(scrubbedPoint ?? defaultPoint, true);
    } else {
      onScrubChange(null, false);
    }
  }, [scrubbing, scrubbedPoint, defaultPoint, onScrubChange]);

  const isolateTouch = useCallback((event: React.TouchEvent) => {
    event.stopPropagation();
  }, []);

  const scrubFromClientX = useCallback(
    (clientX: number) => {
      const el = chartAreaRef.current;
      if (!el) return;
      const point = scrubPointFromClientX(clientX, el.getBoundingClientRect(), points, xDomain, {
        left: yAxisWidth + CHART_MARGIN.left,
        right: CHART_MARGIN.right,
      });
      if (!point) return;
      scrubbingRef.current = true;
      setScrubbing(true);
      setScrubbedPoint(point);
    },
    [points, xDomain, yAxisWidth]
  );

  const handleChartMove = useCallback(
    (state: MouseHandlerDataParam) => {
      const point = pointFromChartState(state, points);
      if (!point) return;
      scrubbingRef.current = true;
      setScrubbing(true);
      setScrubbedPoint(point);
    },
    [points]
  );

  const endScrub = useCallback(() => {
    scrubbingRef.current = false;
    setScrubbing(false);
    setScrubbedPoint(null);
  }, []);

  const handlePointerDown = useCallback(
    (event: React.PointerEvent<HTMLDivElement>) => {
      event.preventDefault();
      event.currentTarget.setPointerCapture(event.pointerId);
      scrubFromClientX(event.clientX);
    },
    [scrubFromClientX]
  );

  const handlePointerMove = useCallback(
    (event: React.PointerEvent<HTMLDivElement>) => {
      if (!scrubbingRef.current && event.pointerType === 'mouse' && event.buttons === 0) {
        return;
      }
      scrubFromClientX(event.clientX);
    },
    [scrubFromClientX]
  );

  const handlePointerUp = useCallback(
    (event: React.PointerEvent<HTMLDivElement>) => {
      if (event.currentTarget.hasPointerCapture(event.pointerId)) {
        event.currentTarget.releasePointerCapture(event.pointerId);
      }
      endScrub();
    },
    [endScrub]
  );

  if (points.length === 0 || !defaultPoint) {
    if (embedded) return null;
    return (
      <section
        className={cn(
          'mobile-panel rounded-2xl border border-border bg-bg-panel/60 p-4',
          className
        )}
      >
        <h3 className="text-sm font-semibold text-fg">{title}</h3>
        <p className="mt-2 text-xs text-fg-muted">No chartable balance history yet.</p>
      </section>
    );
  }

  const chartBody = (
    <>
      {!embedded && (
        <>
          <div className="flex items-center gap-2">
            <LineChartIcon className="h-4 w-4 text-accent" aria-hidden />
            <h3 className="text-sm font-semibold text-fg">{title}</h3>
          </div>
          {caption && <p className="mt-1 text-[11px] text-fg-muted">{caption}</p>}
        </>
      )}

      <div className={cn(embedded ? 'mt-3' : 'mt-3', blurClass)}>
        {!embedded && (
          <PinnedChartValue
            point={displayPoint}
            scrubbing={scrubbing}
            coin={coin}
            ticker={ticker}
          />
        )}

        <div
          ref={chartAreaRef}
          data-chart-scrub
          className={cn(
            'w-full min-w-0 select-none touch-none',
            embedded ? 'mt-1 h-36' : 'mt-2 h-52'
          )}
          style={{ touchAction: 'none' }}
          onTouchStart={isolateTouch}
          onTouchMove={isolateTouch}
          onPointerDown={handlePointerDown}
          onPointerMove={handlePointerMove}
          onPointerUp={handlePointerUp}
          onPointerCancel={handlePointerUp}
        >
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart
              data={points}
              margin={CHART_MARGIN}
              onMouseMove={handleChartMove}
              onMouseLeave={endScrub}
            >
              <defs>
                <linearGradient id="cumulativeBalanceFill" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor="var(--accent)" stopOpacity={0.35} />
                  <stop offset="100%" stopColor="var(--accent)" stopOpacity={0.02} />
                </linearGradient>
              </defs>
              <CartesianGrid
                strokeDasharray="3 3"
                stroke="var(--border)"
                vertical={false}
                syncWithTicks
              />
              <XAxis
                dataKey="time"
                type="number"
                domain={xDomain}
                tickLine={false}
                axisLine={{ stroke: 'var(--border)' }}
                tick={{ fontSize: 10, fill: 'var(--fg-muted)' }}
                tickMargin={8}
                minTickGap={28}
                tickCount={4}
                tickFormatter={(value) => formatChartPointDate(Number(value))}
              />
              <YAxis
                tickLine={false}
                axisLine={false}
                width={yAxisWidth}
                domain={domain}
                tickCount={5}
                tick={{ fontSize: 10, fill: 'var(--fg-muted)' }}
                tickMargin={4}
                tickFormatter={formatChartAxisBalance}
              />
              <Tooltip
                cursor={{
                  stroke: 'var(--accent)',
                  strokeWidth: 1,
                  strokeDasharray: '4 4',
                  strokeOpacity: 0.65,
                }}
                content={() => null}
                wrapperStyle={{ display: 'none' }}
                isAnimationActive={false}
              />
              <Area
                type="monotone"
                dataKey="balance"
                stroke="var(--accent)"
                strokeWidth={2}
                fill="url(#cumulativeBalanceFill)"
                dot={false}
                activeDot={{
                  r: 4,
                  fill: 'var(--accent)',
                  stroke: 'var(--bg-panel)',
                  strokeWidth: 2,
                }}
                isAnimationActive={false}
              />
            </AreaChart>
          </ResponsiveContainer>
        </div>

        {!embedded && (
          <p className="mt-1.5 text-[10px] text-fg-subtle">
            Drag across the chart to inspect history.
          </p>
        )}
      </div>
    </>
  );

  if (embedded) {
    return <div className={cn('border-t border-border/60 pt-3', className)}>{chartBody}</div>;
  }

  return (
    <section
      className={cn('mobile-panel rounded-2xl border border-border bg-bg-panel/60 p-4', className)}
    >
      {chartBody}
    </section>
  );
}
