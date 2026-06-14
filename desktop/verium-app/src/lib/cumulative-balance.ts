import type { TransactionItem } from '@/lib/rpc/client';
import type { CumulativeBalancePoint, CumulativeBalanceSeries } from '@/lib/indexer-api';
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
  if (value === 0) return '0';
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

const SECONDS_PER_DAY = 86_400;

/** Balance at or before `unixSeconds` on a time-sorted cumulative series. */
export function balanceAtOrBefore(
  points: ChartCumulativePoint[],
  unixSeconds: number
): number | null {
  if (points.length === 0) return null;
  let last: ChartCumulativePoint | null = null;
  for (const point of points) {
    if (point.time <= unixSeconds) {
      last = point;
    } else {
      break;
    }
  }
  return last?.balance ?? null;
}

export type BalanceDayChangeTone = 'up' | 'down' | 'flat';

export function balanceDayOverDayChange(
  points: ChartCumulativePoint[],
  currentBalance: number,
  referenceUnixSeconds: number
): { delta: number; priorBalance: number | null; tone: BalanceDayChangeTone } | null {
  if (points.length === 0 || !Number.isFinite(currentBalance)) return null;
  const priorBalance = balanceAtOrBefore(points, referenceUnixSeconds - SECONDS_PER_DAY);
  if (priorBalance == null) return null;
  const delta = currentBalance - priorBalance;
  const tone: BalanceDayChangeTone = delta > 1e-8 ? 'up' : delta < -1e-8 ? 'down' : 'flat';
  return { delta, priorBalance, tone };
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

/** Signed balance effect when walking history backward from the current anchor. */
function walletTxBalanceEffect(tx: TransactionItem): number {
  const amount = tx.amount;
  if (!Number.isFinite(amount)) return 0;
  if (tx.category === 'send') {
    return amount < 0 ? amount : -Math.abs(amount);
  }
  if (tx.category === 'receive' || tx.category === 'immature' || tx.category === 'generate') {
    return amount > 0 ? amount : Math.abs(amount);
  }
  return amount;
}

/**
 * Net balance change for one on-chain tx (all listtransactions rows grouped by txid).
 * When send and receive rows share a txid, change outputs are not treated as new funds.
 */
export function netTxBalanceEffectForGroup(rows: TransactionItem[]): number {
  let sendTotal = 0;
  let recvTotal = 0;
  let other = 0;

  for (const tx of rows) {
    const effect = walletTxBalanceEffect(tx);
    if (tx.category === 'send') {
      sendTotal += effect;
    } else if (
      tx.category === 'receive' ||
      tx.category === 'immature' ||
      tx.category === 'generate'
    ) {
      recvTotal += effect;
    } else {
      other += effect;
    }
  }

  if (sendTotal !== 0 && recvTotal !== 0) {
    return sendTotal + Math.min(recvTotal, Math.abs(sendTotal)) + other;
  }

  return sendTotal + recvTotal + other;
}

function groupTransactionsByTxid(
  txs: TransactionItem[]
): { txid: string; time: number; effect: number }[] {
  const groups = new Map<string, TransactionItem[]>();
  for (const tx of txs) {
    const list = groups.get(tx.txid) ?? [];
    list.push(tx);
    groups.set(tx.txid, list);
  }

  return [...groups.entries()]
    .map(([txid, rows]) => {
      const time = Math.max(...rows.map(transactionTimestamp));
      return { txid, time, effect: netTxBalanceEffectForGroup(rows) };
    })
    .filter((g) => g.time > 0)
    .sort((a, b) => a.time - b.time);
}

function utcDayStart(time: number): number {
  const d = new Date(time * 1000);
  return Math.floor(Date.UTC(d.getUTCFullYear(), d.getUTCMonth(), d.getUTCDate()) / 1000);
}

/** Collapse intraday points to one balance per UTC day (last balance that day). */
function bucketDailyPoints(points: ChartCumulativePoint[]): ChartCumulativePoint[] {
  if (points.length === 0) return points;

  const byDay = new Map<number, ChartCumulativePoint>();
  for (const point of [...points].sort((a, b) => a.time - b.time)) {
    const day = utcDayStart(point.time);
    byDay.set(day, {
      ...point,
      id: `day-${day}`,
      time: day,
      label: formatChartPointDate(day),
    });
  }

  return [...byDay.values()].sort((a, b) => a.time - b.time);
}

/** Historical points should not exceed the live wallet anchor when tx amounts are partial. */
function clampWalletCumulativePoints(
  points: ChartCumulativePoint[],
  anchorBalanceCoins: number
): ChartCumulativePoint[] {
  if (!Number.isFinite(anchorBalanceCoins) || anchorBalanceCoins < 0) {
    return points;
  }
  const cap = anchorBalanceCoins;
  return points.map((p) => ({
    ...p,
    balance: Math.min(Math.max(p.balance, 0), cap),
  }));
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

/** Wallet cumulative balance — daily totals from inception when history is complete. */
export function buildWalletCumulativeSeries(
  txs: TransactionItem[],
  anchorBalanceCoins: number,
  maxPoints = 48
): { points: ChartCumulativePoint[]; complete: boolean; txCountUsed: number } {
  const grouped = groupTransactionsByTxid(txs);
  const complete = txs.length < TRANSACTIONS_LIST_CAP;

  let raw: ChartCumulativePoint[] = [];

  if (complete) {
    if (grouped.length > 0) {
      const firstDay = utcDayStart(grouped[0].time);
      raw.push({
        id: 'origin',
        time: firstDay - 86_400,
        balance: 0,
        label: formatChartPointDate(firstDay - 86_400),
      });
    }

    let balance = 0;
    for (const tx of grouped) {
      balance += tx.effect;
      balance = Math.max(0, balance);
      raw.push({
        id: tx.txid,
        time: tx.time,
        balance,
        label: formatChartPointDate(tx.time, true),
      });
    }
  } else {
    let balance = anchorBalanceCoins;
    const newestFirst = [...grouped].reverse();

    for (const tx of newestFirst) {
      raw.push({
        id: tx.txid,
        time: tx.time,
        balance,
        label: formatChartPointDate(tx.time, true),
      });
      balance -= tx.effect;
      balance = Math.max(0, balance);
    }

    raw.sort((a, b) => a.time - b.time);
  }

  const daily = bucketDailyPoints(raw.filter((p) => p.id !== 'anchor-now'));
  const withAnchor = appendAnchorPoint(daily, anchorBalanceCoins);
  const sampled = downsampleCumulativePoints(withAnchor, maxPoints);
  const points = complete ? sampled : clampWalletCumulativePoints(sampled, anchorBalanceCoins);

  return { points, complete, txCountUsed: txs.length };
}

/** Indexer wallet series → daily chart points (aggregate across all addresses). */
export function indexerWalletSeriesToChart(
  series: CumulativeBalanceSeries,
  anchorBalanceCoins: number,
  maxPoints = 48
): { points: ChartCumulativePoint[]; complete: boolean; txCountUsed: number } {
  let raw: ChartCumulativePoint[] = series.points
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

  if (series.complete && raw.length > 0) {
    const firstDay = utcDayStart(raw[0].time);
    raw = [
      {
        id: 'origin',
        time: firstDay - 86_400,
        balance: 0,
        label: formatChartPointDate(firstDay - 86_400),
      },
      ...raw,
    ];
  }

  const daily = bucketDailyPoints(raw);
  const withAnchor = appendAnchorPoint(daily, anchorBalanceCoins);
  const sampled = downsampleCumulativePoints(withAnchor, maxPoints);
  const points = series.complete
    ? sampled
    : clampWalletCumulativePoints(sampled, anchorBalanceCoins);

  return {
    points,
    complete: series.complete,
    txCountUsed: series.txCountUsed,
  };
}

export function cumulativeSeriesCaption(
  complete: boolean,
  txCountUsed: number,
  txCountTotal?: number | null,
  walletAggregate = false,
  addressMeta?: { truncated?: boolean; used?: number; total?: number }
): string {
  let caption: string;
  if (walletAggregate) {
    if (complete && txCountTotal != null && txCountTotal > 0) {
      caption = `Daily total balance across all wallet addresses since inception (${txCountTotal.toLocaleString()} indexed movements).`;
    } else if (complete) {
      caption = `Daily total balance across all wallet addresses since inception (${txCountUsed.toLocaleString()} indexed movements).`;
    } else {
      const totalLabel =
        txCountTotal != null && txCountTotal > txCountUsed
          ? ` of ${txCountTotal.toLocaleString()} indexed`
          : '';
      caption = `Daily total balance over the latest ${txCountUsed.toLocaleString()} indexed movements${totalLabel} (earlier history not shown).`;
    }
  } else if (complete && txCountTotal != null && txCountTotal > 0) {
    caption = `Daily wallet balance since inception (${txCountTotal.toLocaleString()} transactions).`;
  } else if (complete) {
    caption = `Daily wallet balance since inception (${txCountUsed.toLocaleString()} transactions).`;
  } else {
    const totalLabel =
      txCountTotal != null && txCountTotal > txCountUsed
        ? ` of ${txCountTotal.toLocaleString()} indexed`
        : '';
    caption = `Daily balance over the latest ${txCountUsed.toLocaleString()} transactions${totalLabel} (earlier history not shown).`;
  }

  if (addressMeta?.truncated && addressMeta.used != null && addressMeta.total != null) {
    caption += ` Includes ${addressMeta.used.toLocaleString()} of ${addressMeta.total.toLocaleString()} wallet addresses (cap 1,000).`;
  }

  return caption;
}
