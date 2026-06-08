import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Cpu, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { Badge } from "@/components/ui/Badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/Card";
import { AnimatedHashrate } from "@/components/AnimatedHashrate";
import {
  detectPoolMiner,
  fetchPoolMinerStatus,
  startPoolMiner,
  stopPoolMiner,
} from "@/lib/pool-miner-api";
import { POOL_STRATUM_URL, poolWorkerUsername } from "@/lib/verium-pool";
import { MiningThreadControls } from "@/components/MiningThreadControls";
import { PoolPayoutAddressControls } from "@/components/pool/PoolPayoutAddressControls";
import { poolPayoutAddressConfigured } from "@/lib/pool-dashboard-address";
import type { CpuTopology } from "@/lib/mining-opt";
import { cn } from "@/lib/utils";

export function PoolMiningControls({
  payoutAddress,
  onPayoutAddressChange,
  workerName,
  onWorkerNameChange,
  threads,
  autoAdjustThreads,
  manualThreads,
  suggestedThreads,
  maxThreads,
  scratchpadMib,
  usesSidecar,
  topology,
  logicalCpus,
  onAutoAdjustChange,
  onManualThreadsChange,
  chainSynced,
  syncStalled,
  nodeRpcConnected,
  onStartPool,
  onStopSolo,
}: {
  payoutAddress: string;
  onPayoutAddressChange: (address: string) => void;
  workerName: string;
  onWorkerNameChange: (name: string) => void;
  threads: number;
  autoAdjustThreads: boolean;
  manualThreads: number;
  suggestedThreads?: number;
  maxThreads: number;
  scratchpadMib?: number;
  usesSidecar?: boolean;
  topology?: CpuTopology;
  logicalCpus?: number;
  onAutoAdjustChange: (checked: boolean) => void;
  onManualThreadsChange: (threads: number) => void;
  chainSynced: boolean;
  syncStalled: boolean;
  /** True when veriumd RPC is up (avoids probing while the node is still booting). */
  nodeRpcConnected: boolean;
  onStartPool: () => void;
  onStopSolo: () => void;
}) {
  const queryClient = useQueryClient();

  const detect = useQuery({
    queryKey: ["pool-miner", "detect"],
    queryFn: detectPoolMiner,
    enabled: nodeRpcConnected,
    staleTime: 60_000,
    refetchInterval: (query) => {
      if (!nodeRpcConnected) return false;
      const d = query.state.data;
      if (d?.found && !d.rpcReady) return 10_000;
      return false;
    },
  });

  const status = useQuery({
    queryKey: ["pool-miner", "status"],
    queryFn: fetchPoolMinerStatus,
    enabled: nodeRpcConnected && (detect.data?.rpcReady ?? false),
    refetchInterval: false,
    gcTime: 60_000,
  });

  const running = status.data?.running ?? false;
  const poolMinerBundled = detect.data?.found ?? false;
  const poolMinerRpcReady = detect.data?.rpcReady ?? false;
  const poolMinerReady = poolMinerRpcReady;
  const backendLabel = running || poolMinerRpcReady
    ? "native"
    : poolMinerBundled
      ? "restart node"
      : "upgrade node";
  const username = poolWorkerUsername(payoutAddress, workerName || "wallet");

  const start = useMutation({
    mutationFn: async () => {
      onStopSolo();
      await startPoolMiner({
        stratumUrl: POOL_STRATUM_URL,
        username,
        password: "x",
        threads,
      });
    },
    onSuccess: () => {
      onStartPool();
      void queryClient.invalidateQueries({
        queryKey: ["pool-miner", "status"],
      });
      void queryClient.invalidateQueries({ queryKey: ["pool-miner", "logs"] });
    },
  });

  const stop = useMutation({
    mutationFn: stopPoolMiner,
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: ["pool-miner", "status"],
      });
      void queryClient.invalidateQueries({ queryKey: ["pool-miner", "logs"] });
    },
  });

  const canStart =
    poolMinerReady &&
    chainSynced &&
    !syncStalled &&
    poolPayoutAddressConfigured({ pool_payout_address: payoutAddress }) &&
    !running &&
    !start.isPending;

  return (
    <Card className={cn(running && "ring-1 ring-accent/25")}>
      <CardHeader>
        <div className="flex flex-wrap items-center gap-2">
          <Cpu className="h-4 w-4 text-accent" aria-hidden />
          <CardTitle className="normal-case">Mine on the public pool</CardTitle>
          {running ? (
            <Badge tone="success">Mining</Badge>
          ) : (
            <Badge tone="neutral">Stopped</Badge>
          )}
          <Badge
            tone={
              running || poolMinerRpcReady
                ? "success"
                : poolMinerBundled
                  ? "warning"
                  : "warning"
            }
          >
            {backendLabel}
          </Badge>
        </div>
        <CardDescription>
          {poolMinerRpcReady || running ? (
            <>
              Pool mining runs inside veriumd (Stratum to{" "}
              <span className="font-mono text-xs">{POOL_STRATUM_URL}</span>) —
              same SIMD path as solo mining, no separate miner binary.
            </>
          ) : poolMinerBundled ? (
            <>
              A pool-capable veriumd is bundled, but the running node is still
              on an older build. Stop and restart the Verium node (or restart
              the wallet) to enable in-process pool mining.
            </>
          ) : (
            <>
              This veriumd build does not support in-process pool mining.
              Rebuild veriumd from the latest sources and restart the Verium
              node.
            </>
          )}
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <PoolPayoutAddressControls
          address={payoutAddress}
          disabled={running}
          onAddressChange={onPayoutAddressChange}
        />

        <div className="grid gap-3 sm:grid-cols-2">
          <label className="flex flex-col gap-1 text-sm">
            <span className="text-fg-muted">Worker name</span>
            <input
              value={workerName}
              onChange={(e) => onWorkerNameChange(e.target.value)}
              disabled={running}
              placeholder="wallet"
              className="h-9 rounded-md border border-border bg-bg-panel px-3 font-mono text-sm outline-none focus:border-accent disabled:opacity-60"
            />
          </label>
          <label className="flex flex-col gap-1 text-sm">
            <span className="text-fg-muted">Pool username</span>
            <code className="truncate rounded-md border border-border bg-bg-subtle px-3 py-2 font-mono text-xs">
              {username}
            </code>
          </label>
        </div>

        <MiningThreadControls
          autoAdjust={autoAdjustThreads}
          manualThreads={manualThreads}
          suggestedThreads={suggestedThreads}
          maxThreads={maxThreads}
          topology={topology}
          logicalCpus={logicalCpus}
          activeThreads={
            running ? (status.data?.activeThreads ?? threads) : undefined
          }
          isMining={running}
          liveAdaptive={false}
          disabled={running}
          memoryNote={
            poolMinerRpcReady
              ? "Each thread uses ~128 MiB scrypt scratchpad inside veriumd — thread count is limited to logical CPUs minus one."
              : poolMinerBundled
                ? "Restart the Verium node to load the bundled pool-capable veriumd."
                : scratchpadMib != null
                  ? "Upgrade veriumd to enable native pool mining."
                  : undefined
          }
          onAutoAdjustChange={onAutoAdjustChange}
          onManualThreadsChange={onManualThreadsChange}
        />

        <div className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="text-xs uppercase tracking-wide text-fg-subtle">
              Local hashrate
            </p>
            <p className="text-2xl font-bold tabular-nums">
              {running ? (
                <AnimatedHashrate
                  value={status.data?.hashrateHm}
                  fractionDigits={2}
                  immediate={running}
                />
              ) : (
                <AnimatedHashrate value={0} fractionDigits={2} />
              )}
            </p>
          </div>

          <div className="flex flex-wrap gap-2">
            {running ? (
              <Button
                type="button"
                variant="danger"
                disabled={stop.isPending}
                onClick={() => stop.mutate()}
              >
                {stop.isPending ? (
                  <Loader2 className="h-4 w-4 animate-spin" aria-hidden />
                ) : null}
                Stop pool mining
              </Button>
            ) : (
              <Button
                type="button"
                variant="primary"
                disabled={!canStart}
                onClick={() => start.mutate()}
              >
                {start.isPending ? (
                  <Loader2 className="h-4 w-4 animate-spin" aria-hidden />
                ) : null}
                Start pool mining
              </Button>
            )}
          </div>
        </div>

        {start.error ? (
          <p className="text-sm text-danger">{String(start.error.message)}</p>
        ) : null}
        {!chainSynced && !syncStalled ? (
          <p className="text-sm text-warning">
            Wait for the node to sync before starting the pool miner.
          </p>
        ) : null}
      </CardContent>
    </Card>
  );
}

export function usePoolMinerRunning(nodeRpcConnected = false): boolean {
  const status = useQuery({
    queryKey: ["pool-miner", "status"],
    queryFn: fetchPoolMinerStatus,
    enabled: nodeRpcConnected,
    refetchInterval: false,
    gcTime: 60_000,
  });
  return status.data?.running ?? false;
}
