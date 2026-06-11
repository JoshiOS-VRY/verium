import { BootstrapBanner } from "@/components/BootstrapBanner";
import { BackupHealthCard } from "@/components/BackupHealthCard";
import { DashboardHero } from "@/components/DashboardHero";
import { ExplorerRecentBlocks } from "@/components/ExplorerRecentBlocks";
import { MobileBalanceHero } from "@/components/mobile/MobileBalanceHero";
import { MobileDashboardHero } from "@/components/mobile/MobileDashboardHero";
import { MobileQuickActions } from "@/components/mobile/MobileQuickActions";
import { useActiveCoin } from "@/lib/coin/context";
import { useIsTestNetwork } from "@/lib/network-mode";
import { useWalletMode } from "@/hooks/useWalletMode";
import { LightWalletDashboardUnlock } from "@/components/LightWalletDashboardUnlock";
import { LightWalletMissingBanner } from "@/components/LightWalletMissingBanner";
import { LightWalletSyncBanner } from "@/components/LightWalletSyncBanner";

export function Dashboard() {
  const coin = useActiveCoin();
  const isTestNetwork = useIsTestNetwork();
  const { isLight, mobileOnly } = useWalletMode();

  if (mobileOnly) {
    return (
      <div className="mobile-page">
        {isLight && <LightWalletMissingBanner />}
        {isLight && <LightWalletDashboardUnlock />}
        {isLight && <LightWalletSyncBanner />}
        <MobileBalanceHero />
        <MobileQuickActions />
        <MobileDashboardHero coin={coin} />
        {!isTestNetwork && (
          <ExplorerRecentBlocks coin={coin} variant="dashboard" />
        )}
      </div>
    );
  }

  return (
    <div className="flex min-w-0 max-w-full flex-col gap-5 sm:gap-4 xl:gap-4">
      {isLight && <LightWalletMissingBanner />}
      {isLight && <LightWalletDashboardUnlock />}
      {isLight && <LightWalletSyncBanner />}
      {!isLight && <BootstrapBanner />}
      <DashboardHero coin={coin} />
      {!isTestNetwork && (
        <ExplorerRecentBlocks coin={coin} variant="dashboard" />
      )}
      <BackupHealthCard />
    </div>
  );
}
