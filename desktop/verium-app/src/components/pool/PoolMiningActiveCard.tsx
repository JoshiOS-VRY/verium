import { useQuery } from "@tanstack/react-query";
import { Cpu } from "lucide-react";
import { AnimatedHashrate } from "@/components/AnimatedHashrate";
import { Badge } from "@/components/ui/Badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/Card";
import { fetchPoolMinerStatus } from "@/lib/pool-miner-api";

/** Lightweight pool view while veriumMiner runs — no Supabase or Recharts. */
export function PoolMiningActiveCard() {
  const status = useQuery({
    queryKey: ["pool-miner", "status"],
    queryFn: fetchPoolMinerStatus,
    refetchInterval: false,
  });

  const running = status.data?.running ?? false;
  const backend = status.data?.backend;
  const threads = status.data?.activeThreads;

  return (
    <Card className="ring-1 ring-accent/25">
      <CardHeader className="pb-2">
        <div className="flex flex-wrap items-center gap-2">
          <Cpu className="h-4 w-4 text-accent" aria-hidden />
          <CardTitle className="normal-case text-base">Pool mining active</CardTitle>
          <Badge tone="success">Live</Badge>
          {backend ? (
            <Badge tone={backend === "veriumMiner" ? "success" : "warning"}>
              {backend === "veriumMiner" ? "veriumMiner" : "native"}
            </Badge>
          ) : null}
        </div>
        <CardDescription>
          Cloud pool stats and charts pause while veriumMiner runs to keep wallet
          memory low. Local hashrate updates every ~15s.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <p className="text-xs uppercase tracking-wide text-fg-subtle">
          Local hashrate
        </p>
        <p className="text-3xl font-bold tabular-nums">
          {running ? (
            <AnimatedHashrate
              value={status.data?.hashrateHm}
              fractionDigits={2}
              immediate
            />
          ) : (
            "—"
          )}
        </p>
        {threads != null && threads > 0 ? (
          <p className="mt-2 text-sm text-fg-muted">
            {threads} thread{threads === 1 ? "" : "s"} · stop mining to refresh
            pool dashboard
          </p>
        ) : null}
      </CardContent>
    </Card>
  );
}
