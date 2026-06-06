import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { nodeStateChannelCount } from "@/lib/node-state-listener";

export interface MemoryDiagnostics {
  walletProcessRssBytes: number;
  backgroundTasks: number;
  nodeStateListenerCount: number;
  buildProfile: string;
}

const TELEMETRY_INTERVAL_MS = 60_000;

function formatMb(bytes: number): string {
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/** Dev-only periodic memory telemetry (console + optional state for diagnostics UI). */
export function useMemoryTelemetry(enabled = import.meta.env.DEV): MemoryDiagnostics | null {
  const [diag, setDiag] = useState<MemoryDiagnostics | null>(null);

  useEffect(() => {
    if (!enabled) return;

    let cancelled = false;

    const sample = async () => {
      try {
        const raw = await invoke<MemoryDiagnostics>("get_memory_diagnostics");
        if (cancelled) return;
        setDiag(raw);

        const perfMemory = (
          performance as Performance & { memory?: { usedJSHeapSize: number } }
        ).memory;
        const heap = perfMemory?.usedJSHeapSize;

        console.debug("[memory]", {
          walletRss: formatMb(raw.walletProcessRssBytes),
          jsHeap: heap != null ? formatMb(heap) : "n/a",
          backgroundTasks: raw.backgroundTasks,
          nodeStateChannels: nodeStateChannelCount(),
          nodeStateListeners: raw.nodeStateListenerCount,
        });
      } catch {
        /* non-tauri or command unavailable */
      }
    };

    void sample();
    const id = window.setInterval(() => void sample(), TELEMETRY_INTERVAL_MS);
    return () => {
      cancelled = true;
      window.clearInterval(id);
    };
  }, [enabled]);

  return diag;
}
