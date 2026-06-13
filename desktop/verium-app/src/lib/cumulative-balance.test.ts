import { describe, expect, it } from 'vitest';
import {
  balanceAtOrBefore,
  balanceDayOverDayChange,
  buildWalletCumulativeSeries,
  computeChartBalanceDomain,
  downsampleCumulativePoints,
  formatChartAxisBalance,
  indexerWalletSeriesToChart,
  netTxBalanceEffectForGroup,
} from './cumulative-balance';
import type { CumulativeBalanceSeries } from './indexer-api';
import type { TransactionItem } from '@/lib/rpc/client';

describe('cumulative balance chart', () => {
  it('starts at zero and ends at anchor when history is complete', () => {
    const txs: TransactionItem[] = [
      {
        txid: 'b',
        category: 'receive',
        amount: 9.4,
        confirmations: 100,
        time: 1_600_000_000,
        timereceived: 1_600_000_000,
      },
      {
        txid: 'a',
        category: 'immature',
        amount: 290,
        confirmations: 10,
        time: 1_700_000_000,
        timereceived: 1_700_000_000,
      },
    ];

    const series = buildWalletCumulativeSeries(txs, 299.4);
    expect(series.points[0]?.balance).toBe(0);
    expect(series.points[series.points.length - 1]?.balance).toBe(299.4);
    expect(Math.max(...series.points.map((p) => p.balance))).toBe(299.4);
  });

  it('formats axis labels for large balances', () => {
    expect(formatChartAxisBalance(0)).toBe('0');
    expect(formatChartAxisBalance(300)).toBe('300');
    expect(formatChartAxisBalance(1234)).toBe('1.2k');
    expect(formatChartAxisBalance(9.4)).toBe('9.4');
  });

  it('includes anchor in chart domain', () => {
    const domain = computeChartBalanceDomain(
      [{ id: '1', time: 1, balance: 9.4, label: 'Jan 1' }],
      300
    );
    expect(domain[1]).toBeGreaterThan(300);
  });

  it('nets self-send payment and change rows by txid', () => {
    const rows: TransactionItem[] = [
      {
        txid: 'self',
        category: 'send',
        amount: -1,
        confirmations: 10,
        time: 1_700_000_000,
        timereceived: 1_700_000_000,
      },
      {
        txid: 'self',
        category: 'receive',
        amount: 1,
        confirmations: 10,
        time: 1_700_000_000,
        timereceived: 1_700_000_000,
      },
      {
        txid: 'self',
        category: 'receive',
        amount: 279,
        confirmations: 10,
        time: 1_700_000_000,
        timereceived: 1_700_000_000,
      },
    ];
    expect(netTxBalanceEffectForGroup(rows)).toBe(0);

    const txs: TransactionItem[] = [
      ...rows,
      {
        txid: 'fund',
        category: 'receive',
        amount: 280,
        confirmations: 100,
        time: 1_600_000_000,
        timereceived: 1_600_000_000,
      },
    ];

    const series = buildWalletCumulativeSeries(txs, 280);
    expect(series.points[0]?.balance).toBe(0);
    expect(series.points[series.points.length - 1]?.balance).toBe(280);
  });

  it('does not exceed anchor when send rows have inflated outgoing amounts', () => {
    const txs: TransactionItem[] = [
      {
        txid: 'send',
        category: 'send',
        amount: -160,
        confirmations: 10,
        time: 1_700_000_100,
        timereceived: 1_700_000_100,
      },
      {
        txid: 'recv',
        category: 'receive',
        amount: 300,
        confirmations: 100,
        time: 1_600_000_000,
        timereceived: 1_600_000_000,
      },
    ];

    const series = buildWalletCumulativeSeries(txs, 140);
    expect(Math.max(...series.points.map((p) => p.balance))).toBe(300);
    expect(series.points[series.points.length - 1]?.balance).toBe(140);
  });

  it('does not show negative history when send rows have wrong positive amounts', () => {
    const txs: TransactionItem[] = [
      {
        txid: 'send',
        category: 'send',
        amount: 299,
        confirmations: 0,
        time: 1_700_000_100,
        timereceived: 1_700_000_100,
      },
      {
        txid: 'recv',
        category: 'receive',
        amount: 300,
        confirmations: 100,
        time: 1_600_000_000,
        timereceived: 1_600_000_000,
      },
    ];

    const series = buildWalletCumulativeSeries(txs, 1);
    expect(Math.min(...series.points.map((p) => p.balance))).toBeGreaterThanOrEqual(0);
  });

  it('downsample keeps extrema', () => {
    const points = Array.from({ length: 100 }, (_, i) => ({
      id: String(i),
      time: i,
      balance: i === 50 ? 500 : i,
      label: String(i),
    }));
    const sampled = downsampleCumulativePoints(points, 12);
    expect(sampled.some((p) => p.balance === 500)).toBe(true);
    expect(sampled[0]?.balance).toBe(0);
    expect(sampled[sampled.length - 1]?.balance).toBe(99);
  });

  it('aggregates indexer wallet series into daily totals ending at anchor', () => {
    const day1 = 1_600_000_000;
    const day2 = day1 + 86_400;
    const series: CumulativeBalanceSeries = {
      points: [
        {
          time: day1,
          blockHeight: 100,
          balance: { amount: '10.00000000', ticker: 'VRM', decimalPlaces: 8 },
        },
        {
          time: day2,
          blockHeight: 200,
          balance: { amount: '25.00000000', ticker: 'VRM', decimalPlaces: 8 },
        },
      ],
      currentBalance: { amount: '30.00000000', ticker: 'VRM', decimalPlaces: 8 },
      txCountUsed: 2,
      txCountTotal: 2,
      complete: true,
    };

    const chart = indexerWalletSeriesToChart(series, 30);
    expect(chart.points[0]?.balance).toBe(0);
    expect(chart.points[chart.points.length - 1]?.balance).toBe(30);
    expect(chart.points.some((p) => p.balance === 25)).toBe(true);
  });

  it('computes day-over-day balance change from chart points', () => {
    const now = 1_700_000_000;
    const yesterday = now - 86_400;
    const points = [
      { id: 'a', time: yesterday - 3600, balance: 10, label: 'old' },
      { id: 'b', time: yesterday, balance: 12, label: 'yesterday' },
      { id: 'c', time: now, balance: 15, label: 'now' },
    ];
    expect(balanceAtOrBefore(points, yesterday)).toBe(12);
    const up = balanceDayOverDayChange(points, 15, now);
    expect(up?.delta).toBe(3);
    expect(up?.tone).toBe('up');
    const flat = balanceDayOverDayChange(
      [
        { id: 'y', time: yesterday, balance: 12, label: 'yesterday' },
        { id: 'n', time: now, balance: 12, label: 'now' },
      ],
      12,
      now
    );
    expect(flat?.delta).toBe(0);
    expect(flat?.tone).toBe('flat');
  });
});
