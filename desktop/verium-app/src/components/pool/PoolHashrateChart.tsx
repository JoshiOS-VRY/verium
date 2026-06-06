import {
  Area,
  AreaChart,
  CartesianGrid,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/Card";
import type { HashratePoint } from "@/lib/pool-api";
import { fmtHashrate, fmtHashrateAxis, hsToHm } from "@/lib/pool-format";

export function PoolHashrateChart({
  points,
  loading,
}: {
  points: HashratePoint[];
  loading?: boolean;
}) {
  const data = points.map((p) => ({
    t: new Date(p.ts).getTime(),
    hm: hsToHm(p.hashrate),
  }));

  return (
    <Card>
      <CardHeader>
        <CardTitle className="normal-case">Pool hashrate (12h)</CardTitle>
        <CardDescription>30m smoothed · 30s buckets · all workers</CardDescription>
      </CardHeader>
      <CardContent>
        {loading && data.length === 0 ? (
          <p className="py-8 text-center text-sm text-fg-muted">Loading chart…</p>
        ) : data.length === 0 ? (
          <p className="py-8 text-center text-sm text-fg-muted">
            No share history yet. Connect a miner to the pool to see hashrate.
          </p>
        ) : (
          <div className="h-48 w-full overflow-hidden">
            <AreaChart width={720} height={192} data={data}>
                <CartesianGrid strokeDasharray="3 3" className="stroke-border" />
                <XAxis
                  dataKey="t"
                  type="number"
                  domain={["dataMin", "dataMax"]}
                  tickFormatter={(v) =>
                    new Date(v).toLocaleTimeString(undefined, {
                      hour: "2-digit",
                      minute: "2-digit",
                    })
                  }
                  tick={{ fontSize: 10 }}
                />
                <YAxis
                  tickFormatter={(v) => fmtHashrateAxis((v as number) / 60)}
                  tick={{ fontSize: 10 }}
                  width={48}
                />
                <Tooltip
                  labelFormatter={(v) => new Date(v as number).toLocaleString()}
                  formatter={(value) => [
                    fmtHashrate((value as number) / 60),
                    "Hashrate",
                  ]}
                />
                <Area
                  type="monotone"
                  dataKey="hm"
                  stroke="var(--accent)"
                  fill="var(--accent)"
                  fillOpacity={0.15}
                />
              </AreaChart>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
