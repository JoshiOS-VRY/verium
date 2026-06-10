import { useEffect, useMemo, useRef, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { coinQueryKey } from "@/lib/coin/profile";
import { PoolMiningControls } from "@/components/pool/PoolMiningControls";
import {
  MiningHashrateChart,
  type HashSample,
} from "@/components/MiningHashrateChart";
import { ExternalLinkButton } from "@/components/ExternalLinkButton";
import {
  Card,
  CardHeader,
  CardTitle,
} from "@/components/ui/Card";
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

export function PoolMiningPanel({
  prefs,
  enabled,
  payoutAddress,
  workerName,
  onPoolIdentityChange,
  miningThreads,
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
  onStartPool,
  onStopSolo,
}: {
  prefs: UserPreferences;
  enabled: boolean;
  payoutAddress: string;
  workerName: string;
  onPoolIdentityChange: (payoutAddress: string, workerName: string) => void;
  miningThreads: number;
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
  onStartPool: () => void;
  onStopSolo: () => void;
}) {
  const updatePrefs = useUserPreferences((s) => s.update);
  const prefsLoaded = useUserPreferences((s) => s.loaded);

  const addresses = useQuery({
    queryKey: coinQueryKey("verium", "listaddressgroupings"),
    queryFn: () => rpcListAddressGroupings("verium"),
    enabled,
    staleTime: 30_000,
  });

  const dashboardAddress = resolvePoolDashboardAddress(prefs, addresses.data);
  const dashboardHref = dashboardAddress
    ? poolMinerUrl(dashboardAddress)
    : POOL_WEB_URL;

  const localStatus = useQuery({
    queryKey: ["pool-miner", "status"],
    queryFn: fetchPoolMinerStatus,
    enabled,
    refetchInterval: false,
  });
  const localPoolMining = localStatus.data?.running === true;
  const localHashrate = localStatus.data?.hashrateHm ?? 0;

  // Session hashrate samples for the chart, sourced from the (supervisor-backed)
  // pool miner status poll. Reset whenever a mining session ends.
  const [samples, setSamples] = useState<HashSample[]>([]);
  const [sessionStartedAt, setSessionStartedAt] = useState<number>();
  const lastSampleRef = useRef<{ t: number; hr: number } | null>(null);

  useEffect(() => {
    if (!localPoolMining) {
      lastSampleRef.current = null;
      setSamples([]);
      setSessionStartedAt(undefined);
      return;
    }
    setSessionStartedAt((prev) => prev ?? Math.floor(Date.now() / 1000));
  }, [localPoolMining]);

  useEffect(() => {
    if (!localPoolMining) return;
    const now = Date.now();
    const last = lastSampleRef.current;
    if (last && localHashrate === last.hr && now - last.t < 4000) return;
    lastSampleRef.current = { t: now, hr: localHashrate };
    setSamples((prev) =>
      [...prev, { t: now, hashrate: localHashrate }].slice(-60),
    );
  }, [localHashrate, localPoolMining]);

  const sessionAvg = useMemo(() => {
    if (samples.length === 0) return localPoolMining ? localHashrate : null;
    return samples.reduce((a, s) => a + s.hashrate, 0) / samples.length;
  }, [samples, localPoolMining, localHashrate]);

  useEffect(() => {
    // Wait until persisted prefs are loaded — otherwise we may overwrite a saved
    // payout address with a first-run suggestion while defaults are still showing.
    if (!prefsLoaded) return;
    if (prefs.pool_payout_address?.trim()) return;
    const suggested = suggestPoolPayoutAddress(prefs, addresses.data);
    if (suggested) {
      void updatePrefs({ pool_payout_address: suggested });
    }
  }, [
    prefsLoaded,
    prefs.pool_payout_address,
    prefs.mining_reward_address_mode,
    prefs.mining_reward_address,
    addresses.data,
    updatePrefs,
  ]);

  return (
    <div className="flex flex-col gap-4">
      <PoolMiningControls
        payoutAddress={payoutAddress}
        workerName={workerName}
        onPoolIdentityChange={onPoolIdentityChange}
        threads={miningThreads}
        autoAdjustThreads={autoAdjustThreads}
        manualThreads={manualThreads}
        suggestedThreads={suggestedThreads}
        maxThreads={maxThreads}
        topology={topology}
        logicalCpus={logicalCpus}
        onAutoAdjustChange={onAutoAdjustChange}
        onManualThreadsChange={onManualThreadsChange}
        chainSynced={chainSynced}
        syncStalled={syncStalled}
        nodeRpcConnected={enabled}
        onStartPool={onStartPool}
        onStopSolo={onStopSolo}
      />

      {localPoolMining ? (
        <MiningHashrateChart
          samples={samples}
          sessionAvg={sessionAvg}
          sessionStartedAt={sessionStartedAt}
          active={localPoolMining}
        />
      ) : null}

      <Card>
        <CardHeader className="flex-row items-center justify-between gap-3">
          <CardTitle className="normal-case">Pool dashboard</CardTitle>
          <ExternalLinkButton href={dashboardHref} variant="primary">
            Open pool.vericonomy.com
          </ExternalLinkButton>
        </CardHeader>
      </Card>
    </div>
  );
}
