import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/Card";
import { CopyButton } from "@/components/pool/CopyButton";
import {
  POOL_FEE_PERCENT,
  POOL_MIN_PAYOUT_CONFIRMATIONS,
  POOL_PAYOUT_THRESHOLD_VRM,
  POOL_STRATUM_URL,
  poolWorkerUsername,
} from "@/lib/verium-pool";

export function PoolSetupCard({
  address,
  workerName = "desktop",
}: {
  address: string;
  workerName?: string;
}) {
  const username = poolWorkerUsername(address, workerName);
  const cmd = `cpuminer -a scrypt2 \\
  -o ${POOL_STRATUM_URL} \\
  -u ${username} \\
  -p x \\
  -t <threads>`;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="normal-case">Connect to the pool</CardTitle>
        <CardDescription>
          Use the Start pool mining button above for the built-in miner. Advanced
          users can also point an external scrypt² CPU miner at these settings.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <div>
          <p className="mb-1 text-xs font-medium text-fg-muted">Stratum URL</p>
          <div className="flex flex-wrap items-center gap-2">
            <code className="block flex-1 rounded-md border border-border bg-bg-subtle px-3 py-2 font-mono text-xs">
              {POOL_STRATUM_URL}
            </code>
            <CopyButton value={POOL_STRATUM_URL} label="Copy stratum URL" />
          </div>
        </div>
        <div>
          <p className="mb-1 text-xs font-medium text-fg-muted">
            Username (address.worker)
          </p>
          <div className="flex flex-wrap items-center gap-2">
            <code className="block flex-1 rounded-md border border-border bg-bg-subtle px-3 py-2 font-mono text-xs">
              {username}
            </code>
            <CopyButton value={username} label="Copy miner username" />
          </div>
        </div>
        <pre className="overflow-x-auto rounded-md border border-border bg-bg-subtle p-3 font-mono text-xs text-fg-muted">
          {cmd}
        </pre>
        <p className="text-xs text-fg-subtle">
          Pool fee {POOL_FEE_PERCENT}% · PPLNS · min payout {POOL_PAYOUT_THRESHOLD_VRM}{" "}
          VRM · {POOL_MIN_PAYOUT_CONFIRMATIONS} confirmations
        </p>
      </CardContent>
    </Card>
  );
}
