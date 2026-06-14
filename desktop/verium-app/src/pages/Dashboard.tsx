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
import { useResponsiveLayout } from '@/hooks/useResponsiveLayout';
import { useWindowVisible } from '@/hooks/useWindowVisible';
import { LightWalletDashboardUnlock } from '@/components/LightWalletDashboardUnlock';
import { LightWalletMissingBanner } from '@/components/LightWalletMissingBanner';
import { LightWalletSyncBanner } from '@/components/LightWalletSyncBanner';
import { MobilePullToRefresh } from '@/components/mobile/MobilePullToRefresh';
import { refreshMobileDashboard } from '@/lib/dashboard-refresh';

export function Dashboard() {
  const coin = useActiveCoin();
  const isTestNetwork = useIsTestNetwork();
  const { isLight, isPhoneLayout } = useResponsiveLayout();
  const visible = useWindowVisible();
  const queryClient = useQueryClient();

  useLightWalletInstantReceiveSync(coin, isLight && visible);

  const { refreshing, runRefresh } = useMobileBalanceRefreshing();

  const handleMobileRefresh = useCallback(async () => {
    await runRefresh(() => refreshMobileDashboard(queryClient, coin, isLight));
  }, [runRefresh, queryClient, coin, isLight]);

  if (isPhoneLayout) {
    return (
      <div className="mobile-page mobile-page--dashboard">
        <div className="mobile-dashboard-span-full">
          <MobilePullToRefresh onRefresh={handleMobileRefresh} />
        </div>
        {isLight && (
          <div className="mobile-dashboard-span-full">
            <LightWalletMissingBanner />
          </div>
        )}
        {isLight && (
          <div className="mobile-dashboard-span-full">
            <LightWalletDashboardUnlock />
          </div>
        )}
        <div className="mobile-dashboard-primary">
          <MobileBalanceHero showChart refreshing={refreshing} />
        </div>
        <div className="mobile-dashboard-secondary">
          <MobileQuickActions className="h-full" />
        </div>
        {!isTestNetwork && (
          <div className="mobile-dashboard-span-full">
            <ExplorerRecentBlocks coin={coin} variant="dashboard" />
          </div>
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
      {!isTestNetwork && <ExplorerRecentBlocks coin={coin} variant="dashboard" />}
      <BackupHealthCard />
    </div>
  );
}
