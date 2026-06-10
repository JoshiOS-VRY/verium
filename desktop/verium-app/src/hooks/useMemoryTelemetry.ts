import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useQueryClient } from "@tanstack/react-query";
import { nodeStateChannelCount } from "@/lib/node-state-listener";
import {
  heapGrowthMb,
  heapSampleCount,
  recordHeapSample,
  takeFrontendSnapshot,
  type FrontendMemorySnapshot,
} from "@/lib/memory-profiler";

export interface MemoryDiagnostics {
  walletProcessRssBytes: number;
  backgroundTasks: number;
  nodeStateListenerCount: number;
  buildProfile: string;
  rpcCallCount: number;
  uptimeSecs: number;
}

export interface MemoryTelemetryState {
  rust: MemoryDiagnostics;
  frontend: FrontendMemorySnapshot;
  heapGrowthMb: number | null;
  heapSampleCount: number;
}

const TELEMETRY_INTERVAL_MS = 60_000;

function formatMb(bytes: number): string {
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/** Dev-only periodic memory telemetry (console + optional state for diagnostics UI). */
export function useMemoryTelemetry(
  enabled = import.meta.env.DEV,
): MemoryTelemetryState | null {
  const queryClient = useQueryClient();
  const [state, setState] = useState<MemoryTelemetryState | null>(null);

  useEffect(() => {
    if (!enabled) return;

    let cancelled = false;

    const sample = async () => {
      try {
        const raw = await invoke<MemoryDiagnostics>("get_memory_diagnostics");
        if (cancelled) return;

        recordHeapSample();
        const frontend = takeFrontendSnapshot(queryClient);
        const next: MemoryTelemetryState = {
          rust: raw,
          frontend,
          heapGrowthMb: heapGrowthMb(),
          heapSampleCount: heapSampleCount(),
        };
        setState(next);

        const rpcPerMin =
          raw.uptimeSecs > 0
            ? ((raw.rpcCallCount / raw.uptimeSecs) * 60).toFixed(1)
            : "n/a";
        console.debug("[memory]", {
          walletRss: formatMb(raw.walletProcessRssBytes),
          jsHeap:
            frontend.jsHeapUsedMb != null
              ? `${frontend.jsHeapUsedMb.toFixed(1)} MB`
              : "n/a",
          heapGrowth: next.heapGrowthMb != null ? `${next.heapGrowthMb.toFixed(2)} MB` : "n/a",
          queryCache: frontend.queryCacheEntries,
          queryObservers: frontend.queryCacheObservers,
          backgroundTasks: raw.backgroundTasks,
          nodeStateChannels: nodeStateChannelCount(),
          nodeStateListeners: raw.nodeStateListenerCount,
          rpcCalls: raw.rpcCallCount,
          rpcCallsPerMin: rpcPerMin,
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
  }, [enabled, queryClient]);

  return state;
}
