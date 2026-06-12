import { describe, expect, it } from 'vitest';
import type { ChartCumulativePoint } from '@/lib/cumulative-balance';
import { nearestPointByTime, scrubPointFromClientX } from '@/lib/chart-scrub';

describe('chart-scrub', () => {
  const points: ChartCumulativePoint[] = [
    { id: 'a', time: 100, balance: 1, label: '100' },
    { id: 'b', time: 200, balance: 2, label: '200' },
    { id: 'c', time: 300, balance: 3, label: '300' },
  ];

  it('picks the nearest point by time', () => {
    expect(nearestPointByTime(points, 190)?.balance).toBe(2);
    expect(nearestPointByTime(points, 110)?.balance).toBe(1);
  });

  it('maps client X across the plot width', () => {
    const rect = { left: 0, right: 200, top: 0, bottom: 100, width: 200, height: 100 } as DOMRect;
    const point = scrubPointFromClientX(188, rect, points, [100, 300], { left: 40, right: 12 });
    expect(point?.balance).toBe(3);
  });
});
