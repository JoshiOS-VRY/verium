import { BootstrapBanner } from "@/components/BootstrapBanner";
import { BackupHealthCard } from "@/components/BackupHealthCard";
import { DashboardHero } from "@/components/DashboardHero";
import { DashboardMiddleRow } from "@/components/DashboardMiddleRow";
import { ExplorerRecentBlocks } from "@/components/ExplorerRecentBlocks";
import { PoolStatsStrip } from "@/components/pool/PoolStatsStrip";
import { useActiveCoin } from "@/lib/coin/context";
import { useIsTestNetwork } from "@/lib/network-mode";
import { useWalletMode } from "@/hooks/useWalletMode";
import { LightWalletDashboardUnlock } from "@/components/LightWalletDashboardUnlock";
import { LightWalletSyncBanner } from "@/components/LightWalletSyncBanner";

export function Dashboard() {
  const coin = useActiveCoin();
  const isTestNetwork = useIsTestNetwork();
  const { isLight } = useWalletMode();

  return (
    <div className="flex flex-col gap-4">
      {isLight && <LightWalletDashboardUnlock />}
      {isLight && <LightWalletSyncBanner />}
      {!isLight && <BootstrapBanner />}
      <DashboardHero coin={coin} />
      <DashboardMiddleRow coin={coin} />
      {coin === "verium" && !isTestNetwork && <PoolStatsStrip />}
      {!isTestNetwork && (
        <ExplorerRecentBlocks coin={coin} variant="dashboard" />
      )}
      <BackupHealthCard />
    </div>
  );
}
