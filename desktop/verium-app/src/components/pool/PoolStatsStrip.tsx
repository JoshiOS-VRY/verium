import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { AnimatedHashrate } from "@/components/AnimatedHashrate";
import { AnimatedNumber } from "@/components/AnimatedNumber";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/Card";
import { StatCell, StatGrid } from "@/components/ui/StatGrid";
import { ExternalLinkButton } from "@/components/ExternalLinkButton";
import { useWindowVisible } from "@/hooks/useWindowVisible";
import { usePoolStatsQuery } from "@/hooks/usePoolQueries";
import { hsToHm } from "@/lib/pool-format";
import { POOL_DASHBOARD_POLL_MS } from "@/lib/mining-poll";
import { POOL_WEB_URL } from "@/lib/verium-pool";
import { timeAgo } from "@/lib/pool-format";

export function PoolStatsStrip({ enabled = true }: { enabled?: boolean }) {
  const visible = useWindowVisible();
  const queryClient = useQueryClient();
  const stats = usePoolStatsQuery(enabled && visible);

  useEffect(() => {
    if (!enabled || !visible) return;
    const tick = () => {
      void queryClient.invalidateQueries({ queryKey: ["pool", "stats"] });
    };
    tick();
    const id = window.setInterval(tick, POOL_DASHBOARD_POLL_MS);
    return () => window.clearInterval(id);
  }, [enabled, visible, queryClient]);

  const s = stats.data;
  const poolHm =
    s?.poolHashrate != null ? hsToHm(s.poolHashrate) : undefined;

  return (
    <Card>
      <CardHeader className="flex-row items-center justify-between gap-3">
        <CardTitle className="normal-case">Official Verium pool</CardTitle>
        <ExternalLinkButton href={POOL_WEB_URL}>pool.vericonomy.com</ExternalLinkButton>
      </CardHeader>
      <CardContent>
        {stats.isError && !s ? (
          <p className="text-sm text-fg-muted">
            Pool stats unavailable. Build with POOL_SUPABASE_ANON_KEY or check network.
          </p>
        ) : (
          <>
            <StatGrid className="lg:grid-cols-5">
              <StatCell
                label="Pool hashrate"
                value={
                  <AnimatedHashrate value={poolHm} fractionDigits={1} />
                }
              />
              <StatCell
                label="Active miners"
                value={
                  <AnimatedNumber
                    value={s?.activeMiners ?? undefined}
                    fractionDigits={0}
                  />
                }
              />
              <StatCell
                label="Active workers"
                value={
                  <AnimatedNumber
                    value={s?.activeWorkers ?? undefined}
                    fractionDigits={0}
                  />
                }
              />
              <StatCell
                label="Blocks found"
                value={
                  <AnimatedNumber
                    value={s?.blocksFoundTotal ?? undefined}
                    fractionDigits={0}
                  />
                }
              />
              <StatCell
                label="Pool fee"
                value={
                  s?.poolFeePct != null ? (
                    <span>{s.poolFeePct}%</span>
                  ) : (
                    "—"
                  )
                }
              />
            </StatGrid>
            {s?.lastBlockAt ? (
              <p className="mt-4 text-xs text-fg-subtle">
                Last block found {timeAgo(s.lastBlockAt)}
              </p>
            ) : null}
          </>
        )}
      </CardContent>
    </Card>
  );
}
