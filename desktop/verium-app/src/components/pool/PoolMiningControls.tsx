import { useEffect, useRef, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Cpu, Loader2 } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Badge } from '@/components/ui/Badge';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { AnimatedHashrate } from '@/components/AnimatedHashrate';
import {
  detectPoolMiner,
  fetchPoolMinerStatus,
  startPoolMiner,
  stopPoolMiner,
} from '@/lib/pool-miner-api';
import { POOL_STRATUM_BACKUP_URL, POOL_STRATUM_URL, poolWorkerUsername } from '@/lib/verium-pool';
import { MiningThreadControls } from '@/components/MiningThreadControls';
import { PoolPayoutAddressControls } from '@/components/pool/PoolPayoutAddressControls';
import { poolPayoutAddressConfigured } from '@/lib/pool-dashboard-address';
import { normalizePoolPayoutAddress, normalizePoolWorkerName } from '@/lib/pool-mining-prefs';
import type { CpuTopology } from '@/lib/mining-opt';
import { cn } from '@/lib/utils';

const POOL_IDENTITY_SAVE_MS = 450;

function formatPoolMinerError(error: unknown): string {
  if (error instanceof Error && error.message.trim()) return error.message;
  const text = String(error).trim();
  return text && text !== '[object Object]' ? text : 'Pool miner failed to start';
}

export function PoolMiningControls({
  payoutAddress,
  workerName,
  onPoolIdentityChange,
  threads,
  autoAdjustThreads,
  manualThreads,
  suggestedThreads,
  maxThreads,
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
  workerName: string;
  onPoolIdentityChange: (payoutAddress: string, workerName: string) => void;
  threads: number;
  autoAdjustThreads: boolean;
  manualThreads: number;
  suggestedThreads?: number;
  maxThreads: number;
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
    queryKey: ['pool-miner', 'detect'],
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

  const usesSidecar = detect.data?.source === 'sidecar';

  const status = useQuery({
    queryKey: ['pool-miner', 'status'],
    queryFn: fetchPoolMinerStatus,
    enabled: nodeRpcConnected && (usesSidecar || (detect.data?.rpcReady ?? false)),
    refetchInterval: false,
    gcTime: 60_000,
  });

  const running = status.data?.running ?? false;
  const poolMinerBundled = detect.data?.found ?? false;
  const poolMinerRpcReady = detect.data?.rpcReady ?? false;
  const poolMinerReady = usesSidecar || poolMinerRpcReady;
  const acceptedShares = status.data?.acceptedShares ?? 0;
  const rejectedShares = status.data?.rejectedShares ?? 0;
  const totalShares = acceptedShares + rejectedShares;
  const rejectRate = totalShares > 0 ? rejectedShares / totalShares : 0;
  const showRejectWarning = running && totalShares >= 10 && rejectRate > 0.05;

  const [localPayout, setLocalPayout] = useState(payoutAddress);
  const [localWorker, setLocalWorker] = useState(workerName);
  const editingRef = useRef(false);
  const saveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (editingRef.current) return;
    setLocalPayout(payoutAddress);
    setLocalWorker(workerName);
  }, [payoutAddress, workerName]);

  useEffect(() => {
    return () => {
      if (saveTimerRef.current) clearTimeout(saveTimerRef.current);
    };
  }, []);

  const flushIdentity = (address: string, worker: string) => {
    onPoolIdentityChange(normalizePoolPayoutAddress(address), normalizePoolWorkerName(worker));
  };

  const scheduleIdentitySave = (address: string, worker: string) => {
    if (saveTimerRef.current) clearTimeout(saveTimerRef.current);
    saveTimerRef.current = setTimeout(() => {
      saveTimerRef.current = null;
      editingRef.current = false;
      flushIdentity(address, worker);
    }, POOL_IDENTITY_SAVE_MS);
  };

  const applyIdentity = (address: string, worker: string, options?: { immediate?: boolean }) => {
    setLocalPayout(address);
    setLocalWorker(worker);
    if (options?.immediate) {
      if (saveTimerRef.current) clearTimeout(saveTimerRef.current);
      saveTimerRef.current = null;
      editingRef.current = false;
      flushIdentity(address, worker);
      return;
    }
    editingRef.current = true;
    scheduleIdentitySave(address, worker);
  };

  const username = poolWorkerUsername(localPayout, localWorker.trim() || 'wallet');

  const start = useMutation({
    mutationFn: async () => {
      onStopSolo();
      // Persist payout address + worker name before connecting so the next
      // session restores the same pool identity.
      flushIdentity(localPayout, localWorker);
      // IBD-aware cap: while the node is still syncing, leave headroom for the
      // sidecar to mine without starving node validation.
      const effectiveThreads =
        usesSidecar && !chainSynced ? Math.max(1, Math.floor(threads / 2)) : threads;
      await startPoolMiner({
        stratumUrl: POOL_STRATUM_URL,
        username,
        password: 'x',
        threads: effectiveThreads,
        backupUrl: POOL_STRATUM_BACKUP_URL || undefined,
      });
    },
    onSuccess: () => {
      onStartPool();
      void queryClient.invalidateQueries({
        queryKey: ['pool-miner', 'status'],
      });
    },
  });

  const stop = useMutation({
    mutationFn: stopPoolMiner,
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: ['pool-miner', 'status'],
      });
    },
  });

  // The sidecar connects straight to the pool, so it can mine during node IBD;
  // the in-process (native) miner stays gated on a synced node.
  const syncGateOk = usesSidecar || (chainSynced && !syncStalled);
  const canStart =
    poolMinerReady &&
    syncGateOk &&
    poolPayoutAddressConfigured({ pool_payout_address: localPayout }) &&
    !running &&
    !start.isPending;

  return (
    <Card className={cn(running && 'ring-1 ring-accent/25')}>
      <CardHeader>
        <div className="flex flex-wrap items-center gap-2">
          <Cpu className="h-4 w-4 text-accent" aria-hidden />
          <CardTitle className="normal-case">Mine on the public pool</CardTitle>
          {running ? <Badge tone="success">Mining</Badge> : <Badge tone="neutral">Stopped</Badge>}
        </div>
        {!usesSidecar && !poolMinerRpcReady && !running ? (
          <CardDescription>
            {poolMinerBundled
              ? 'Restart the Verium node to enable pool mining.'
              : 'Update veriumd to enable pool mining.'}
          </CardDescription>
        ) : null}
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <PoolPayoutAddressControls
          address={localPayout}
          disabled={running}
          onAddressChange={(addr) => applyIdentity(addr, localWorker, { immediate: true })}
        />

        <div className="grid gap-3 sm:grid-cols-2">
          <label className="flex flex-col gap-1 text-sm">
            <span className="text-fg-muted">Worker name</span>
            <input
              value={localWorker}
              onChange={(e) => applyIdentity(localPayout, e.target.value)}
              onBlur={() => {
                if (saveTimerRef.current) clearTimeout(saveTimerRef.current);
                saveTimerRef.current = null;
                editingRef.current = false;
                flushIdentity(localPayout, localWorker);
              }}
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
          activeThreads={running ? (status.data?.activeThreads ?? threads) : undefined}
          isMining={running}
          liveAdaptive={false}
          disabled={running}
          onAutoAdjustChange={onAutoAdjustChange}
          onManualThreadsChange={onManualThreadsChange}
        />

        <div className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="text-xs uppercase tracking-wide text-fg-subtle">Local hashrate</p>
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
            {usesSidecar && running ? (
              <p className="mt-1 text-xs text-fg-subtle tabular-nums">
                {acceptedShares} accepted · {rejectedShares} rejected
                {status.data?.connectionState ? ` · ${status.data.connectionState}` : ''}
              </p>
            ) : null}
          </div>

          <div className="flex flex-wrap gap-2">
            {running ? (
              <Button
                type="button"
                variant="danger"
                disabled={stop.isPending}
                onClick={() => stop.mutate()}
              >
                {stop.isPending ? <Loader2 className="h-4 w-4 animate-spin" aria-hidden /> : null}
                Stop pool mining
              </Button>
            ) : (
              <Button
                type="button"
                variant="primary"
                disabled={!canStart}
                onClick={() => start.mutate()}
              >
                {start.isPending ? <Loader2 className="h-4 w-4 animate-spin" aria-hidden /> : null}
                Start pool mining
              </Button>
            )}
          </div>
        </div>

        {start.error ? (
          <p className="text-sm text-danger">{formatPoolMinerError(start.error)}</p>
        ) : null}
        {showRejectWarning ? (
          <p className="text-sm text-warning">
            High reject rate ({(rejectRate * 100).toFixed(1)}%). Check your connection or lower the
            thread count if this persists.
          </p>
        ) : null}
        {!usesSidecar && !chainSynced && !syncStalled ? (
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
    queryKey: ['pool-miner', 'status'],
    queryFn: fetchPoolMinerStatus,
    enabled: nodeRpcConnected,
    refetchInterval: false,
    gcTime: 60_000,
  });
  return status.data?.running ?? false;
}
