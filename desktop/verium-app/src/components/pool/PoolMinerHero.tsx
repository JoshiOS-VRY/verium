import { Activity, Cpu } from "lucide-react";
import { Badge } from "@/components/ui/Badge";
import { ExternalLinkButton } from "@/components/ExternalLinkButton";
import { CopyButton } from "@/components/pool/CopyButton";
import { fmtHashrate, fmtRejectPct, timeAgo } from "@/lib/pool-format";
import { poolMinerUrl } from "@/lib/verium-pool";
import { cn } from "@/lib/utils";

function ellipsizeMiddle(s: string, max = 36): string {
  if (s.length <= max) return s;
  const half = Math.floor((max - 1) / 2);
  return `${s.slice(0, half)}…${s.slice(-half)}`;
}

export function PoolMinerHero({
  address,
  lastShareAt,
  currentHashrate,
  avgHashrate,
  workerCount,
  onlineWorkers,
  accepted,
  rejected,
  stale,
}: {
  address: string;
  lastShareAt: string | null;
  currentHashrate: number;
  avgHashrate: number;
  workerCount: number;
  onlineWorkers: number;
  accepted: number;
  rejected: number;
  stale: number;
}) {
  const isLive = onlineWorkers > 0;
  const reject = fmtRejectPct(accepted, rejected, stale);

  return (
    <section
      className={cn(
        "overflow-hidden rounded-lg border border-border bg-bg-panel",
        isLive && "ring-1 ring-accent/20",
      )}
    >
      <div className="border-b border-border/60 px-4 py-3 sm:px-6">
        <div className="flex flex-wrap items-center gap-2">
          <Cpu className="h-4 w-4 text-accent" aria-hidden />
          <span className="text-xs font-semibold uppercase tracking-wider text-fg-subtle">
            Pool miner
          </span>
          {isLive ? (
            <Badge tone="success">Mining</Badge>
          ) : (
            <Badge tone="warning">Offline</Badge>
          )}
          <span className="ml-auto text-xs text-fg-subtle">
            {workerCount} worker{workerCount === 1 ? "" : "s"}
            {isLive ? ` · ${onlineWorkers} online` : ""}
          </span>
        </div>
      </div>

      <div className="flex flex-col gap-6 p-4 sm:flex-row sm:items-end sm:justify-between sm:p-6">
        <div className="min-w-0 flex-1 space-y-4">
          <div>
            <p className="text-xs font-medium uppercase tracking-wide text-fg-muted">
              5m average H/m
            </p>
            <p className="mt-1 text-3xl font-bold tabular-nums text-fg">
              {fmtHashrate(currentHashrate)}
            </p>
            {avgHashrate > 0 ? (
              <p className="mt-2 text-sm text-fg-muted">
                15m avg{" "}
                <span className="font-semibold text-fg">
                  {fmtHashrate(avgHashrate)}
                </span>
              </p>
            ) : null}
          </div>

          <div className="flex flex-wrap items-center gap-2">
            <code className="max-w-full truncate rounded-lg border border-border bg-bg-subtle px-3 py-1.5 font-mono text-sm">
              {ellipsizeMiddle(address)}
            </code>
            <CopyButton value={address} label="Copy wallet address" />
          </div>

          <div className="flex flex-wrap gap-x-5 gap-y-2 text-sm">
            <div className="flex items-center gap-1.5 text-fg-muted">
              <Activity className="h-3.5 w-3.5" aria-hidden />
              Last share{" "}
              <span className="font-semibold text-fg">{timeAgo(lastShareAt)}</span>
            </div>
            <div>
              <span className="text-fg-subtle">Accepted </span>
              <span className="font-semibold">{accepted}</span>
            </div>
            <div>
              <span className="text-fg-subtle">Reject </span>
              <span
                className={cn(
                  "font-semibold",
                  reject.tone === "bad"
                    ? "text-danger"
                    : reject.tone === "warn"
                      ? "text-warning"
                      : "text-success",
                )}
              >
                {reject.label}
              </span>
            </div>
          </div>
        </div>

        <ExternalLinkButton href={poolMinerUrl(address)} variant="primary">
          Open on pool site
        </ExternalLinkButton>
      </div>
    </section>
  );
}
