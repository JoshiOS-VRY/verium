import { useMemoryTelemetry } from "@/hooks/useMemoryTelemetry";
import { nodeStateChannelCount } from "@/lib/node-state-listener";

function formatMb(bytes: number): string {
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/** Fixed dev overlay — does not affect production builds. */
export function MemoryDiagnosticsPanel() {
  const diag = useMemoryTelemetry();

  if (!import.meta.env.DEV || !diag) return null;

  const perfMemory = (
    performance as Performance & { memory?: { usedJSHeapSize: number } }
  ).memory;
  const heap = perfMemory?.usedJSHeapSize;

  return (
    <div
      className="pointer-events-none fixed bottom-2 right-2 z-[9999] max-w-xs rounded border border-border/60 bg-bg/90 px-2 py-1.5 font-mono text-[10px] leading-snug text-fg-muted shadow-sm backdrop-blur-sm"
      aria-hidden
    >
      <div>RSS {formatMb(diag.walletProcessRssBytes)}</div>
      {heap != null ? <div>JS heap {formatMb(heap)}</div> : null}
      <div>bg tasks {diag.backgroundTasks}</div>
      <div>node channels {nodeStateChannelCount()}</div>
      <div>node listeners {diag.nodeStateListenerCount}</div>
    </div>
  );
}
