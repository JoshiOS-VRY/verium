import { useEffect, useRef, useState } from "react";
import { ScrollText } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/Card";
import { Badge } from "@/components/ui/Badge";
import { useWindowVisible } from "@/hooks/useWindowVisible";
import {
  fetchPoolMinerLogLines,
  fetchPoolMinerStatus,
} from "@/lib/pool-miner-api";
import { cn } from "@/lib/utils";

export function PoolMinerLogPanel() {
  const visible = useWindowVisible();
  const containerRef = useRef<HTMLDivElement>(null);
  const [stickToBottom, setStickToBottom] = useState(true);
  const [liveLogsEnabled, setLiveLogsEnabled] = useState(false);

  const status = useQuery({
    queryKey: ["pool-miner", "status"],
    queryFn: fetchPoolMinerStatus,
    refetchInterval: false,
  });

  const running = status.data?.running ?? false;
  const backend = status.data?.backend ?? "";
  const logs = useQuery({
    queryKey: ["pool-miner", "logs"],
    queryFn: () => fetchPoolMinerLogLines(40),
    // Live logs are expensive in WebView2: keep them opt-in and bounded.
    enabled: running && visible && liveLogsEnabled,
    refetchInterval: visible && running && liveLogsEnabled ? 5_000 : false,
    gcTime: 15_000,
  });
  const lines = logs.data ?? [];
  const logText = lines.join("\n");

  useEffect(() => {
    if (!stickToBottom) return;
    const el = containerRef.current;
    if (!el) return;
    // Avoid repeated smooth-scroll animations; they are expensive in WebView2.
    el.scrollTop = el.scrollHeight;
  }, [lines, stickToBottom]);

  const onScroll = () => {
    const el = containerRef.current;
    if (!el) return;
    const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 32;
    setStickToBottom((prev) => (prev === atBottom ? prev : atBottom));
  };

  return (
    <Card className={cn(running && "ring-1 ring-accent/20")}>
      <CardHeader className="pb-2">
        <div className="flex flex-wrap items-center gap-2">
          <ScrollText className="h-4 w-4 text-accent" aria-hidden />
          <CardTitle className="normal-case text-base">Miner log</CardTitle>
          {running ? (
            <Badge tone="success">Live</Badge>
          ) : (
            <Badge tone="neutral">Idle</Badge>
          )}
          {backend ? (
            <Badge tone={backend === "veriumMiner" ? "success" : "warning"}>
              {backend === "veriumMiner" ? "veriumMiner" : "native"}
            </Badge>
          ) : null}
          {running ? (
            <button
              type="button"
              className="rounded border border-border px-2 py-0.5 text-xs text-fg-muted hover:text-fg"
              onClick={() => setLiveLogsEnabled((v) => !v)}
            >
              {liveLogsEnabled ? "Pause logs" : "Show live logs"}
            </button>
          ) : null}
        </div>
        <CardDescription>
          Output from the pool miner process. Hashrate is polled separately from
          the miner API.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div
          ref={containerRef}
          onScroll={onScroll}
          className="h-44 overflow-auto rounded-md border border-border bg-bg-subtle/80 p-3 font-mono text-[11px] leading-relaxed text-fg-muted"
        >
          {!running ? (
            <p className="text-fg-subtle">
              Start pool mining to stream veriumMiner logs here.
            </p>
          ) : !liveLogsEnabled ? (
            <p className="text-fg-subtle">
              Live log streaming is paused to reduce wallet memory/CPU usage.
            </p>
          ) : lines.length === 0 ? (
            <p className="text-fg-subtle">Waiting for miner output…</p>
          ) : (
            <pre className="whitespace-pre-wrap break-all">{logText}</pre>
          )}
        </div>
        {!stickToBottom && lines.length > 0 ? (
          <p className="mt-2 text-xs text-fg-subtle">
            Scroll paused — scroll to the bottom to resume auto-follow.
          </p>
        ) : null}
      </CardContent>
    </Card>
  );
}
