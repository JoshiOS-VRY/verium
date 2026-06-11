import { BootstrapBanner } from '@/components/BootstrapBanner';
import { BackupHealthCard } from '@/components/BackupHealthCard';
import { DashboardHero } from '@/components/DashboardHero';
import { ExplorerRecentBlocks } from '@/components/ExplorerRecentBlocks';
import { useActiveCoin } from '@/lib/coin/context';
import { useIsTestNetwork } from '@/lib/network-mode';
import { useWalletMode } from '@/hooks/useWalletMode';
import { LightWalletDashboardUnlock } from '@/components/LightWalletDashboardUnlock';
import { LightWalletMissingBanner } from '@/components/LightWalletMissingBanner';
import { LightWalletSyncBanner } from '@/components/LightWalletSyncBanner';

export function Dashboard() {
  const coin = useActiveCoin();
  const isTestNetwork = useIsTestNetwork();
  const { isLight } = useWalletMode();

  return (
    <div className="flex flex-col gap-5 sm:gap-4 xl:gap-4">
      {isLight && <LightWalletMissingBanner />}
      {isLight && <LightWalletDashboardUnlock />}
      {isLight && <LightWalletSyncBanner />}
      {!isLight && <BootstrapBanner />}
      <DashboardHero coin={coin} />
      {!isTestNetwork && <ExplorerRecentBlocks coin={coin} variant="dashboard" />}
      <BackupHealthCard />
    </div>
  );
}
