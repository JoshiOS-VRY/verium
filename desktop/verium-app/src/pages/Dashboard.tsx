import { useCallback } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { BootstrapBanner } from '@/components/BootstrapBanner';
import { BackupHealthCard } from '@/components/BackupHealthCard';
import { DashboardHero } from '@/components/DashboardHero';
import { ExplorerRecentBlocks } from '@/components/ExplorerRecentBlocks';
import {
  MobileBalanceHero,
  useMobileBalanceRefreshing,
} from '@/components/mobile/MobileBalanceHero';
import { MobileQuickActions } from '@/components/mobile/MobileQuickActions';
import { useActiveCoin } from '@/lib/coin/context';
import { useIsTestNetwork } from '@/lib/network-mode';
import { useLightWalletInstantReceiveSync } from '@/hooks/useLightWalletInstantReceiveSync';
import { useWalletMode } from '@/hooks/useWalletMode';
import { useWindowVisible } from '@/hooks/useWindowVisible';
import { LightWalletDashboardUnlock } from '@/components/LightWalletDashboardUnlock';
import { LightWalletMissingBanner } from '@/components/LightWalletMissingBanner';
import { LightWalletSyncBanner } from '@/components/LightWalletSyncBanner';
import { MobilePullToRefresh } from '@/components/mobile/MobilePullToRefresh';
import { refreshMobileDashboard } from '@/lib/dashboard-refresh';

export function Dashboard() {
  const coin = useActiveCoin();
  const isTestNetwork = useIsTestNetwork();
  const { isLight, mobileOnly } = useWalletMode();
  const visible = useWindowVisible();
  const queryClient = useQueryClient();

  useLightWalletInstantReceiveSync(coin, isLight && visible);

  const { refreshing, runRefresh } = useMobileBalanceRefreshing();

  const handleMobileRefresh = useCallback(async () => {
    await runRefresh(() => refreshMobileDashboard(queryClient, coin, isLight));
  }, [runRefresh, queryClient, coin, isLight]);

  if (mobileOnly) {
    return (
      <div className="mobile-page">
        <MobilePullToRefresh onRefresh={handleMobileRefresh} />
        {isLight && <LightWalletMissingBanner />}
        {isLight && <LightWalletDashboardUnlock />}
        {!mobileOnly && isLight && <LightWalletSyncBanner />}
        <MobileBalanceHero showChart refreshing={refreshing} />
        <MobileQuickActions />
        {!isTestNetwork && <ExplorerRecentBlocks coin={coin} variant="dashboard" />}
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
      {!isTestNetwork && <ExplorerRecentBlocks coin={coin} variant="dashboard" />}
      <BackupHealthCard />
    </div>
  );
}
