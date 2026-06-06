import { useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import { coinQueryKey } from "@/lib/coin/profile";
import { PoolHashrateChart } from "@/components/pool/PoolHashrateChart";
import { PoolMinerHero } from "@/components/pool/PoolMinerHero";
import { PoolMiningActiveCard } from "@/components/pool/PoolMiningActiveCard";
import { PoolMinerPayoutsPanel } from "@/components/pool/PoolMinerPayoutsPanel";
import { PoolMinerRewardsPanel } from "@/components/pool/PoolMinerRewardsPanel";
import { PoolMinerWorkersPanel } from "@/components/pool/PoolMinerWorkersPanel";
import { PoolSetupCard } from "@/components/pool/PoolSetupCard";
import { PoolMinerLogPanel } from "@/components/pool/PoolMinerLogPanel";
import { PoolMiningControls } from "@/components/pool/PoolMiningControls";
import { ExternalLinkButton } from "@/components/ExternalLinkButton";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/Card";
import {
  useMinerHashrateHistoryQuery,
  useMinerOverviewQuery,
  useMinerPayoutsQuery,
} from "@/hooks/usePoolQueries";
import { usePoolDashboardPolls } from "@/hooks/usePoolDashboardPolls";
import {
  resolvePoolDashboardAddress,
  suggestPoolPayoutAddress,
} from "@/lib/pool-dashboard-address";
import { fetchPoolMinerStatus } from "@/lib/pool-miner-api";
import { rpcListAddressGroupings } from "@/lib/rpc/client";
import { poolMinerUrl, POOL_WEB_URL } from "@/lib/verium-pool";
import type { UserPreferences } from "@/lib/user-preferences";
import { useUserPreferences } from "@/lib/user-preferences";
import type { CpuTopology } from "@/lib/mining-opt";
import { RejectPctBadge } from "@/components/pool/RejectPctBadge";
import { fmtHashrate, fmtRejectPct } from "@/lib/pool-format";
import { StatCell, StatGrid } from "@/components/ui/StatGrid";

export function PoolMiningPanel({
  prefs,
  enabled,
  payoutAddress,
  onPayoutAddressChange,
  workerName,
  onWorkerNameChange,
  miningThreads,
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
  onStartPool,
  onStopSolo,
}: {
  prefs: UserPreferences;
  enabled: boolean;
  payoutAddress: string;
  onPayoutAddressChange: (address: string) => void;
  workerName: string;
  onWorkerNameChange: (name: string) => void;
  miningThreads: number;
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
  onStartPool: () => void;
  onStopSolo: () => void;
}) {
  const updatePrefs = useUserPreferences((s) => s.update);

  const addresses = useQuery({
    queryKey: coinQueryKey("verium", "listaddressgroupings"),
    queryFn: () => rpcListAddressGroupings("verium"),
    enabled,
    staleTime: 30_000,
  });

  const address = resolvePoolDashboardAddress(prefs, addresses.data);

  const localStatus = useQuery({
    queryKey: ["pool-miner", "status"],
    queryFn: fetchPoolMinerStatus,
    refetchInterval: false,
  });
  const localPoolMining = localStatus.data?.running === true;

  const poolDashboardEnabled = enabled && Boolean(address) && !localPoolMining;
  usePoolDashboardPolls(address, poolDashboardEnabled);

  useEffect(() => {
    if (prefs.pool_payout_address?.trim()) return;
    const suggested = suggestPoolPayoutAddress(prefs, addresses.data);
    if (suggested) {
      void updatePrefs({ pool_payout_address: suggested });
    }
  }, [
    prefs.pool_payout_address,
    prefs.mining_reward_address_mode,
    prefs.mining_reward_address,
    addresses.data,
    updatePrefs,
  ]);

  const overview = useMinerOverviewQuery(address, poolDashboardEnabled);
  const history = useMinerHashrateHistoryQuery(address, poolDashboardEnabled);
  const payouts = useMinerPayoutsQuery(address, poolDashboardEnabled);

  const controls = (
    <PoolMiningControls
      payoutAddress={payoutAddress}
      onPayoutAddressChange={onPayoutAddressChange}
      workerName={workerName}
      onWorkerNameChange={onWorkerNameChange}
      threads={miningThreads}
      autoAdjustThreads={autoAdjustThreads}
      manualThreads={manualThreads}
      suggestedThreads={suggestedThreads}
      maxThreads={maxThreads}
      scratchpadMib={scratchpadMib}
      usesSidecar={usesSidecar}
      topology={topology}
      logicalCpus={logicalCpus}
      onAutoAdjustChange={onAutoAdjustChange}
      onManualThreadsChange={onManualThreadsChange}
      chainSynced={chainSynced}
      syncStalled={syncStalled}
      onStartPool={onStartPool}
      onStopSolo={onStopSolo}
    />
  );

  if (localPoolMining) {
    return (
      <div className="flex flex-col gap-4">
        <PoolMinerLogPanel />
        {controls}
        <PoolMiningActiveCard />
      </div>
    );
  }

  if (!address) {
    return (
      <div className="flex flex-col gap-4">
        <PoolMinerLogPanel />
        {controls}
        <Card>
          <CardHeader>
            <CardTitle className="normal-case">Pool mining</CardTitle>
            <CardDescription>
              Create a receive address in the wallet, then choose it as your pool
              payout address above.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <PoolSetupCard address="VYourAddress" workerName={workerName || "wallet"} />
          </CardContent>
        </Card>
      </div>
    );
  }

  const m = overview.data;
  if (overview.isLoading && !m) {
    return (
      <div className="flex flex-col gap-4">
        <PoolMinerLogPanel />
        {controls}
        <p className="text-sm text-fg-muted">Loading pool dashboard…</p>
      </div>
    );
  }

  if (!m) {
    return (
      <div className="flex flex-col gap-4">
        <PoolMinerLogPanel />
        {controls}
        <PoolSetupCard address={address} workerName={workerName || "wallet"} />
        <Card>
          <CardHeader>
            <CardTitle className="normal-case">No pool shares yet</CardTitle>
            <CardDescription>
              Start pool mining above — shares and payouts will appear here once
              the pool sees your worker.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <ExternalLinkButton href={poolMinerUrl(address)} variant="primary">
              View on pool.vericonomy.com
            </ExternalLinkButton>
          </CardContent>
        </Card>
      </div>
    );
  }

  const workers = m.workers ?? [];
  const totalCurrent = workers.reduce(
    (acc, w) => acc + Number(w.current_hashrate ?? 0),
    0,
  );
  const totalAvg = workers.reduce(
    (acc, w) => acc + Number(w.avg_hashrate ?? 0),
    0,
  );
  const totalAccepted = workers.reduce((acc, w) => acc + w.accepted, 0);
  const totalRejected = workers.reduce((acc, w) => acc + w.rejected, 0);
  const totalStale = workers.reduce((acc, w) => acc + w.stale, 0);
  const onlineWorkers = workers.filter((w) => w.state === "online").length;
  const overallReject = fmtRejectPct(
    totalAccepted,
    totalRejected,
    totalStale,
  );

  const payoutRows =
    payouts.data?.rows?.length
      ? payouts.data.rows
      : m.recent_payouts ?? [];

  return (
    <div className="flex flex-col gap-4">
      <PoolMinerLogPanel />
      {controls}

      <PoolMinerHero
        address={m.address}
        lastShareAt={m.last_share_at}
        currentHashrate={totalCurrent}
        avgHashrate={totalAvg}
        workerCount={workers.length}
        onlineWorkers={onlineWorkers}
        accepted={totalAccepted}
        rejected={totalRejected}
        stale={totalStale}
      />

      <Card>
        <CardContent className="pt-4">
          <StatGrid className="sm:grid-cols-2 lg:grid-cols-4">
            <StatCell
              label="5m average H/m"
              value={fmtHashrate(totalCurrent)}
            />
            <StatCell label="15m average" value={fmtHashrate(totalAvg)} />
            <StatCell
              label="Workers online"
              value={
                <>
                  {onlineWorkers}
                  <span className="text-base font-medium text-fg-muted">
                    {" "}
                    / {workers.length}
                  </span>
                </>
              }
            />
            <StatCell
              label="Rejection rate"
              value={<RejectPctBadge {...overallReject} />}
            />
          </StatGrid>
        </CardContent>
      </Card>

      <div className="grid gap-4 xl:grid-cols-[1fr_min(20rem,100%)]">
        <PoolHashrateChart
          points={history.data ?? []}
          loading={history.isLoading}
        />
        <PoolMinerRewardsPanel
          pendingSat={m.pending_sat}
          lifetimeRewardSat={m.lifetime_reward_sat}
          lifetimePaidSat={m.lifetime_paid_sat}
        />
      </div>

      <PoolMinerWorkersPanel workers={workers} />
      <PoolMinerPayoutsPanel payouts={payoutRows} />

      <div className="flex justify-center">
        <ExternalLinkButton href={POOL_WEB_URL}>
          Full dashboard on pool.vericonomy.com
        </ExternalLinkButton>
      </div>
    </div>
  );
}
