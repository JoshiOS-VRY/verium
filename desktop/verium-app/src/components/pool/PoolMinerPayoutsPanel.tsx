import { Receipt } from "lucide-react";
import { Badge } from "@/components/ui/Badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/Card";
import { ExplorerLink } from "@/components/ExplorerLink";
import type { PoolPayoutRow } from "@/lib/pool-api";
import { fmtVrm, shortHash, timeAgo } from "@/lib/pool-format";

export function PoolMinerPayoutsPanel({
  payouts,
}: {
  payouts: PoolPayoutRow[];
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="normal-case">Recent payouts</CardTitle>
        <CardDescription>On-chain transfers from the pool</CardDescription>
      </CardHeader>
      <CardContent>
        {payouts.length === 0 ? (
          <div className="flex flex-col items-center gap-2 py-10 text-center">
            <Receipt className="h-10 w-10 text-fg-subtle/60" aria-hidden />
            <p className="text-sm text-fg-muted">No payouts yet</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-sm">
              <thead>
                <tr className="border-b border-border text-xs uppercase text-fg-subtle">
                  <th className="px-3 py-2 font-medium">Amount</th>
                  <th className="px-3 py-2 font-medium">TXID</th>
                  <th className="px-3 py-2 font-medium">State</th>
                  <th className="px-3 py-2 font-medium">When</th>
                </tr>
              </thead>
              <tbody>
                {payouts.map((p, i) => (
                  <tr key={p.txid ?? i} className="border-b border-border/60">
                    <td className="px-3 py-2 font-semibold tabular-nums">
                      {fmtVrm(p.amount_sat)} VRM
                    </td>
                    <td className="px-3 py-2 font-mono text-xs">
                      {p.txid ? (
                        <ExplorerLink
                          coin="verium"
                          target={{ kind: "tx", txid: p.txid }}
                          label={shortHash(p.txid)}
                        />
                      ) : (
                        "—"
                      )}
                    </td>
                    <td className="px-3 py-2">
                      <Badge tone="neutral">{p.state}</Badge>
                    </td>
                    <td className="px-3 py-2 text-fg-muted">
                      {timeAgo(p.created_at)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
