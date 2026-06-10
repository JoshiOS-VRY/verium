/**
 * Dev-only frontend memory profiling helpers.
 * Tracks heap samples, React Query cache size, and optional render counts.
 */

import type { QueryClient } from "@tanstack/react-query";
import { nodeStateChannelCount } from "@/lib/node-state-listener";

export interface HeapSample {
  atMs: number;
  usedBytes: number;
}

export interface FrontendMemorySnapshot {
  atMs: number;
  jsHeapUsedMb: number | null;
  queryCacheEntries: number;
  queryCacheObservers: number;
  nodeStateChannels: number;
  renderCounts: Record<string, number>;
}

const MAX_HEAP_SAMPLES = 120;
const heapSamples: HeapSample[] = [];
const renderCounts = new Map<string, number>();

function readJsHeapBytes(): number | null {
  const mem = (
    performance as Performance & { memory?: { usedJSHeapSize: number } }
  ).memory;
  return mem?.usedJSHeapSize ?? null;
}

/** Record a JS heap sample (call from telemetry interval). */
export function recordHeapSample(): HeapSample | null {
  const used = readJsHeapBytes();
  if (used == null) return null;
  const sample: HeapSample = { atMs: Date.now(), usedBytes: used };
  heapSamples.push(sample);
  if (heapSamples.length > MAX_HEAP_SAMPLES) {
    heapSamples.splice(0, heapSamples.length - MAX_HEAP_SAMPLES);
  }
  return sample;
}

/** Increment a component/hook render counter (dev only). */
export function trackRender(label: string): void {
  if (!import.meta.env.DEV) return;
  renderCounts.set(label, (renderCounts.get(label) ?? 0) + 1);
}

export function queryCacheStats(queryClient: QueryClient): {
  entries: number;
  observers: number;
} {
  const cache = queryClient.getQueryCache();
  const queries = cache.getAll();
  let observers = 0;
  for (const q of queries) {
    observers += q.getObserversCount();
  }
  return { entries: queries.length, observers };
}

export function takeFrontendSnapshot(
  queryClient: QueryClient,
): FrontendMemorySnapshot {
  const heap = readJsHeapBytes();
  const { entries, observers } = queryCacheStats(queryClient);
  return {
    atMs: Date.now(),
    jsHeapUsedMb: heap != null ? heap / (1024 * 1024) : null,
    queryCacheEntries: entries,
    queryCacheObservers: observers,
    nodeStateChannels: nodeStateChannelCount(),
    renderCounts: Object.fromEntries(renderCounts),
  };
}

/** Heap growth between first and last sample in the rolling window (MB). */
export function heapGrowthMb(): number | null {
  if (heapSamples.length < 2) return null;
  const first = heapSamples[0].usedBytes;
  const last = heapSamples[heapSamples.length - 1].usedBytes;
  return (last - first) / (1024 * 1024);
}

export function heapSampleCount(): number {
  return heapSamples.length;
}

export function clearProfilerState(): void {
  heapSamples.length = 0;
  renderCounts.clear();
}

/** Export rolling heap samples for benchmark comparison. */
export function exportHeapSamples(): HeapSample[] {
  return [...heapSamples];
}
