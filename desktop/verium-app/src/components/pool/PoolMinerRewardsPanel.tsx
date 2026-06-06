import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/Card";
import { StatCell, StatGrid } from "@/components/ui/StatGrid";
import { fmtVrm } from "@/lib/pool-format";

export function PoolMinerRewardsPanel({
  pendingSat,
  lifetimeRewardSat,
  lifetimePaidSat,
}: {
  pendingSat: string;
  lifetimeRewardSat: string;
  lifetimePaidSat: string;
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="normal-case">Rewards</CardTitle>
      </CardHeader>
      <CardContent>
        <StatGrid className="sm:grid-cols-3">
          <StatCell
            label="Pending"
            value={<span>{fmtVrm(pendingSat)} VRM</span>}
          />
          <StatCell
            label="Lifetime earned"
            value={<span>{fmtVrm(lifetimeRewardSat)} VRM</span>}
          />
          <StatCell
            label="Lifetime paid"
            value={<span>{fmtVrm(lifetimePaidSat)} VRM</span>}
          />
        </StatGrid>
      </CardContent>
    </Card>
  );
}
