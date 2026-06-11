import { useMemoryTelemetry } from '@/hooks/useMemoryTelemetry';

function formatMb(bytes: number): string {
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/** Fixed dev overlay — does not affect production builds. */
export function MemoryDiagnosticsPanel() {
  const telemetry = useMemoryTelemetry();

  if (!import.meta.env.DEV || !telemetry) return null;

  const { rust, frontend, heapGrowthMb, heapSampleCount } = telemetry;

  return (
    <div
      className="pointer-events-none fixed bottom-2 right-2 z-[9999] max-w-xs rounded-md border border-border/80 bg-bg-panel/90 px-3 py-2 font-mono text-[10px] leading-relaxed text-fg-muted shadow-lg backdrop-blur-sm"
      aria-hidden
    >
      <div className="mb-1 font-semibold uppercase tracking-wide text-fg-subtle">Memory (dev)</div>
      <div>RSS: {formatMb(rust.walletProcessRssBytes)}</div>
      <div>
        JS heap: {frontend.jsHeapUsedMb != null ? `${frontend.jsHeapUsedMb.toFixed(1)} MB` : 'n/a'}
      </div>
      <div>
        Heap Δ ({heapSampleCount}):{' '}
        {heapGrowthMb != null
          ? `${heapGrowthMb >= 0 ? '+' : ''}${heapGrowthMb.toFixed(2)} MB`
          : '—'}
      </div>
      <div>
        Query cache: {frontend.queryCacheEntries} / observers: {frontend.queryCacheObservers}
      </div>
      <div>Node listeners: {rust.nodeStateListenerCount}</div>
      <div>
        RPC: {rust.rpcCallCount} ({rust.uptimeSecs}s)
      </div>
    </div>
  );
}
