import { useRef } from 'react';
import { Outlet, useLocation } from 'react-router-dom';
import { isExplorerDetailPath } from '@/lib/explorer-nav';
import { MobileScrollContext } from '@/contexts/MobileScrollContext';
import { useActiveCoin } from '@/lib/coin/context';
import { useMobileInteractiveBack } from '@/hooks/useMobileInteractiveBack';
import { useWalletMode } from '@/hooks/useWalletMode';
import { DashboardNodeActivity } from './DashboardNodeActivity';
import { NodeRecoveryBanner } from './NodeRecoveryBanner';
import { NetworkModeBanner } from './NetworkModeBanner';
import { ShutdownProgressOverlay } from './ShutdownProgressOverlay';
import { SyncStallBanner } from './SyncStallBanner';
import { CoinSwitchOverlay } from './CoinSwitchOverlay';
import { BiometricSetupOfferHost } from './BiometricSetupPrompt';
import { NotificationPermissionOfferHost } from './NotificationPermissionPrompt';
import { MobileHeader } from './MobileHeader';
import { MobileTabBar } from './MobileTabBar';
import { Sidebar } from './Sidebar';
import { TopBar } from './TopBar';

export function AppShell() {
  const coin = useActiveCoin();
  const { isLight, mobileOnly } = useWalletMode();
  const { pathname } = useLocation();
  const hideMobileTabBar = isExplorerDetailPath(pathname);

  const mobileShellRef = useRef<HTMLDivElement>(null);
  const mobileScrollRef = useRef<HTMLElement>(null);
  const { canBack, contentStyle, scrimStyle, isDragging } = useMobileInteractiveBack(
    mobileShellRef,
    mobileOnly
  );

  if (mobileOnly) {
    return (
      <div
        ref={mobileShellRef}
        className="mobile-shell flex h-dvh max-h-dvh w-full max-w-full flex-col overflow-hidden bg-bg text-fg"
      >
        <ShutdownProgressOverlay />
        <BiometricSetupOfferHost />
        <NotificationPermissionOfferHost />
        {canBack && (
          <div
            className="mobile-swipe-back-scrim pointer-events-none absolute inset-0 z-0 bg-black"
            style={scrimStyle}
            aria-hidden
          />
        )}
        <div
          className="relative z-10 flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden bg-bg"
          style={contentStyle}
        >
          <NetworkModeBanner />
          <MobileHeader />
          <MobileScrollContext.Provider value={mobileScrollRef}>
            <main
              ref={mobileScrollRef}
              className="mobile-main relative min-h-0 min-w-0 flex-1 overflow-y-auto overflow-x-hidden"
              style={{
                overscrollBehaviorX: isDragging ? 'none' : undefined,
              }}
            >
              <CoinSwitchOverlay />
              <div className="mx-auto flex w-full min-w-0 max-w-lg flex-col gap-3">
                <Outlet />
              </div>
            </main>
          </MobileScrollContext.Provider>
          {!hideMobileTabBar && <MobileTabBar />}
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-bg text-fg">
      <ShutdownProgressOverlay />
      <Sidebar />
      <div className="flex min-w-0 flex-1 flex-col">
        <NetworkModeBanner />
        <TopBar />
        <main className="relative flex-1 overflow-y-auto px-4 py-5 sm:px-5 sm:py-6 lg:px-6 xl:px-8">
          <CoinSwitchOverlay />
          <div className="mx-auto flex flex-col gap-4">
            {!isLight && <DashboardNodeActivity coin={coin} />}
            {!isLight && <NodeRecoveryBanner />}
            {!isLight && <SyncStallBanner />}
            <Outlet />
          </div>
        </main>
      </div>
    </div>
  );
}
