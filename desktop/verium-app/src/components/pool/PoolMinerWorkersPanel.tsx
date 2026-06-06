import { HardDrive } from "lucide-react";
import { Badge } from "@/components/ui/Badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/Card";
import { RejectPctBadge } from "@/components/pool/RejectPctBadge";
import type { PoolWorker } from "@/lib/pool-api";
import { fmtHashrate, fmtRejectPct, timeAgo } from "@/lib/pool-format";

export function PoolMinerWorkersPanel({ workers }: { workers: PoolWorker[] }) {
  if (workers.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="normal-case">Workers</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col items-center gap-2 py-10 text-center">
          <HardDrive className="h-10 w-10 text-fg-subtle/60" aria-hidden />
          <p className="text-sm text-fg-muted">No workers yet</p>
          <p className="max-w-sm text-xs text-fg-subtle">
            Point a miner at the pool with{" "}
            <code className="font-mono">YOUR_ADDRESS.workerName</code> as the username.
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="normal-case">Workers</CardTitle>
        <CardDescription>{workers.length} registered</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
          {workers.map((w) => {
            const reject = fmtRejectPct(w.accepted, w.rejected, w.stale);
            const online = w.state === "online";
            return (
              <div
                key={w.id ?? w.name}
                className="rounded-lg border border-border bg-bg-subtle/50 p-4"
              >
                <div className="mb-2 flex items-start justify-between gap-2">
                  <p className="font-mono text-sm font-semibold">{w.name}</p>
                  <Badge tone={online ? "success" : "neutral"}>
                    {w.state}
                  </Badge>
                </div>
                <p className="text-lg font-bold tabular-nums">
                  {fmtHashrate(w.current_hashrate)}
                </p>
                <p className="mt-1 text-xs text-fg-subtle">
                  Last share {timeAgo(w.last_share_at)}
                </p>
                <div className="mt-2">
                  <RejectPctBadge {...reject} />
                </div>
              </div>
            );
          })}
        </div>
      </CardContent>
    </Card>
  );
}
