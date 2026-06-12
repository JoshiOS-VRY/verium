import type { TransactionItem } from '@/lib/rpc/client';
import type { CumulativeBalancePoint } from '@/lib/indexer-api';
import { parseIndexerAmountCoins } from '@/lib/indexer-amount';
import { TRANSACTIONS_LIST_CAP } from '@/lib/transactions-list';

export interface ChartCumulativePoint {
  id: string;
  time: number;
  balance: number;
  label: string;
}

export function cumulativePointBalance(point: CumulativeBalancePoint): number {
  return parseIndexerAmountCoins(point.balance);
}

export function formatChartAxisBalance(value: number): string {
  if (!Number.isFinite(value)) return '—';
  const abs = Math.abs(value);
  if (abs >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (abs >= 10_000) return `${(value / 1_000).toFixed(0)}k`;
  if (abs >= 1_000) return `${(value / 1_000).toFixed(1)}k`;
  if (abs >= 100) return value.toFixed(0);
  if (abs >= 1) return value.toFixed(1);
  if (abs >= 0.01) return value.toFixed(2);
  return value.toFixed(4);
}

export function computeChartBalanceDomain(
  points: ChartCumulativePoint[],
  anchorBalance?: number
): [number, number] {
  const values = points.map((p) => p.balance);
  if (anchorBalance != null && Number.isFinite(anchorBalance)) {
    values.push(anchorBalance);
  }
  if (values.length === 0) return [0, 1];

  const min = Math.min(...values);
  const max = Math.max(...values);
  const span = max - min;
  const pad = span > 0 ? span * 0.06 : Math.max(max * 0.06, 0.01);
  const lo = min >= 0 ? Math.max(0, min - pad) : min - pad;
  const hi = max + pad;
  return [lo, hi];
}

export function formatChartPointDate(time: number, withTime = false): string {
  if (time <= 0) return '—';
  const when = new Date(time * 1000);
  if (withTime) {
    return when.toLocaleString(undefined, {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: 'numeric',
      minute: '2-digit',
    });
  }
  return when.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
}

export function downsampleCumulativePoints(
  points: ChartCumulativePoint[],
  maxPoints = 48
): ChartCumulativePoint[] {
  if (points.length <= maxPoints) return points;

  const mustKeepIds = new Set<string>();
  mustKeepIds.add(points[0].id);
  mustKeepIds.add(points[points.length - 1].id);

  let maxPoint = points[0];
  let minPoint = points[0];
  for (const point of points) {
    if (point.balance > maxPoint.balance) maxPoint = point;
    if (point.balance < minPoint.balance) minPoint = point;
  }
  mustKeepIds.add(maxPoint.id);
  mustKeepIds.add(minPoint.id);

  const stride = Math.ceil(points.length / maxPoints);
  const sampled: ChartCumulativePoint[] = [];
  for (let i = 0; i < points.length; i += stride) {
    sampled.push(points[i]);
  }

  for (const id of mustKeepIds) {
    const point = points.find((p) => p.id === id);
    if (point && !sampled.some((p) => p.id === id)) {
      sampled.push(point);
    }
  }

  return sampled.sort((a, b) => a.time - b.time);
}

export function indexerPointsToChart(
  points: CumulativeBalancePoint[],
  maxPoints = 48
): ChartCumulativePoint[] {
  const chartable = points
    .filter((p) => p.time != null)
    .map((p, index) => {
      const time = p.time ?? 0;
      return {
        id: `${time}-${p.blockHeight ?? index}`,
        time,
        balance: cumulativePointBalance(p),
        label: formatChartPointDate(time),
      };
    })
    .sort((a, b) => a.time - b.time);

  return downsampleCumulativePoints(chartable, maxPoints);
}

function transactionTimestamp(tx: TransactionItem): number {
  return tx.blocktime ?? tx.time ?? tx.timereceived ?? 0;
}

function appendAnchorPoint(
  points: ChartCumulativePoint[],
  anchorBalanceCoins: number
): ChartCumulativePoint[] {
  if (!Number.isFinite(anchorBalanceCoins)) return points;

  const now = Math.floor(Date.now() / 1000);
  const last = points[points.length - 1];

  if (!last) {
    return [
      {
        id: 'anchor-now',
        time: now,
        balance: anchorBalanceCoins,
        label: 'Now',
      },
    ];
  }

  if (last.id === 'anchor-now') {
    return [
      ...points.slice(0, -1),
      {
        ...last,
        time: now,
        balance: anchorBalanceCoins,
        label: 'Now',
      },
    ];
  }

  if (now - last.time > 3600 || last.balance !== anchorBalanceCoins) {
    return [
      ...points,
      {
        id: 'anchor-now',
        time: now,
        balance: anchorBalanceCoins,
        label: 'Now',
      },
    ];
  }

  return points;
}

/** Wallet cumulative balance anchored at current total balance (newest txs when capped). */
export function buildWalletCumulativeSeries(
  txs: TransactionItem[],
  anchorBalanceCoins: number,
  maxPoints = 48
): { points: ChartCumulativePoint[]; complete: boolean; txCountUsed: number } {
  const sorted = [...txs].sort((a, b) => transactionTimestamp(a) - transactionTimestamp(b));
  const complete = txs.length < TRANSACTIONS_LIST_CAP;

  let balance = anchorBalanceCoins;
  const newestFirst = [...sorted].reverse();
  const raw: ChartCumulativePoint[] = [];

  for (const tx of newestFirst) {
    const time = transactionTimestamp(tx);
    if (time > 0) {
      raw.push({
        id: tx.txid,
        time,
        balance,
        label: formatChartPointDate(time),
      });
    }
    balance -= tx.amount;
  }

  const withAnchor = appendAnchorPoint(
    raw.sort((a, b) => a.time - b.time),
    anchorBalanceCoins
  );

  const points = downsampleCumulativePoints(withAnchor, maxPoints);

  return { points, complete, txCountUsed: txs.length };
}

export function cumulativeSeriesCaption(
  complete: boolean,
  txCountUsed: number,
  txCountTotal?: number | null
): string {
  if (complete && txCountTotal != null && txCountTotal > 0) {
    return `Balance after each indexed transaction (${txCountTotal.toLocaleString()} total).`;
  }
  if (complete) {
    return `Balance after each indexed transaction (${txCountUsed.toLocaleString()} shown).`;
  }
  const totalLabel =
    txCountTotal != null && txCountTotal > txCountUsed
      ? ` of ${txCountTotal.toLocaleString()} indexed`
      : '';
  return `Balance over the latest ${txCountUsed.toLocaleString()} transactions${totalLabel} (anchored to current balance).`;
}
