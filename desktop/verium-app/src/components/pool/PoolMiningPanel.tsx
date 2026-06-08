import { useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import { coinQueryKey } from "@/lib/coin/profile";
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

  const dashboardAddress = resolvePoolDashboardAddress(prefs, addresses.data);
  const dashboardHref = dashboardAddress
    ? poolMinerUrl(dashboardAddress)
    : POOL_WEB_URL;

  const localStatus = useQuery({
    queryKey: ["pool-miner", "status"],
    queryFn: fetchPoolMinerStatus,
    refetchInterval: false,
  });
  const localPoolMining = localStatus.data?.running === true;

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

  return (
    <div className="flex flex-col gap-4">
      {localPoolMining ? <PoolMinerLogPanel /> : null}

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

      <Card>
        <CardHeader>
          <CardTitle className="normal-case">Pool dashboard</CardTitle>
          <CardDescription>
            Workers, hashrate charts, pending rewards, and payout history are on
            the public pool site — not in the wallet.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <ExternalLinkButton href={dashboardHref} variant="primary">
            Open pool.vericonomy.com
          </ExternalLinkButton>
        </CardContent>
      </Card>
    </div>
  );
}
