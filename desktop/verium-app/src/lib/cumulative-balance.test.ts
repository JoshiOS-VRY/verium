import { describe, expect, it } from 'vitest';
import {
  buildWalletCumulativeSeries,
  computeChartBalanceDomain,
  downsampleCumulativePoints,
  formatChartAxisBalance,
} from './cumulative-balance';
import type { TransactionItem } from '@/lib/rpc/client';

describe('cumulative balance chart', () => {
  it('anchors wallet series at total balance including immature', () => {
    const txs: TransactionItem[] = [
      {
        txid: 'a',
        category: 'immature',
        amount: 290,
        confirmations: 10,
        time: 1_700_000_000,
        timereceived: 1_700_000_000,
      },
      {
        txid: 'b',
        category: 'receive',
        amount: 9.4,
        confirmations: 100,
        time: 1_600_000_000,
        timereceived: 1_600_000_000,
      },
    ];

    const spendableOnly = buildWalletCumulativeSeries(txs, 9.4);
    expect(Math.max(...spendableOnly.points.map((p) => p.balance))).toBeLessThan(20);

    const total = buildWalletCumulativeSeries(txs, 300);
    expect(Math.max(...total.points.map((p) => p.balance))).toBe(300);
    expect(total.points[total.points.length - 1]?.balance).toBe(300);
  });

  it('formats axis labels for large balances', () => {
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
});
