import type { MouseHandlerDataParam } from 'recharts/types/synchronisation/types';
import type { ChartCumulativePoint } from '@/lib/cumulative-balance';

export function nearestPointByTime(
  points: ChartCumulativePoint[],
  time: number
): ChartCumulativePoint | null {
  if (points.length === 0) return null;
  let best = points[0];
  let bestDist = Math.abs(points[0].time - time);
  for (let i = 1; i < points.length; i += 1) {
    const dist = Math.abs(points[i].time - time);
    if (dist < bestDist) {
      best = points[i];
      bestDist = dist;
    }
  }
  return best;
}

export function pointIndexFromChartState(state: MouseHandlerDataParam | undefined): number | null {
  const raw = state?.activeTooltipIndex ?? state?.activeIndex;
  if (typeof raw === 'number' && Number.isInteger(raw) && raw >= 0) return raw;
  return null;
}

export function pointFromChartState(
  state: MouseHandlerDataParam | undefined,
  points: ChartCumulativePoint[]
): ChartCumulativePoint | null {
  const index = pointIndexFromChartState(state);
  if (index != null && index < points.length) return points[index];

  const label = state?.activeLabel;
  if (typeof label === 'number' && Number.isFinite(label)) {
    return nearestPointByTime(points, label);
  }
  if (typeof label === 'string') {
    const parsed = Number(label);
    if (Number.isFinite(parsed)) return nearestPointByTime(points, parsed);
  }
  return null;
}

export function scrubPointFromClientX(
  clientX: number,
  rect: DOMRect,
  points: ChartCumulativePoint[],
  xDomain: [number, number],
  plotInset: { left: number; right: number }
): ChartCumulativePoint | null {
  if (points.length === 0) return null;
  const plotLeft = rect.left + plotInset.left;
  const plotRight = rect.right - plotInset.right;
  const plotWidth = plotRight - plotLeft;
  if (plotWidth <= 0) return null;
  const ratio = Math.min(1, Math.max(0, (clientX - plotLeft) / plotWidth));
  const time = xDomain[0] + ratio * (xDomain[1] - xDomain[0]);
  return nearestPointByTime(points, time);
}
