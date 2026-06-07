import { useMemoryTelemetry } from "@/hooks/useMemoryTelemetry";

/** Fixed dev overlay — does not affect production builds. */
export function MemoryDiagnosticsPanel() {
  const diag = useMemoryTelemetry();

  if (!import.meta.env.DEV || !diag) return null;

  return null;
}
