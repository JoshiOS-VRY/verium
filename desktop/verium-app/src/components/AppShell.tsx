import { Outlet } from "react-router-dom";
import { useActiveCoin } from "@/lib/coin/context";
import { useWalletMode } from "@/hooks/useWalletMode";
import { DashboardNodeActivity } from "./DashboardNodeActivity";
import { NodeRecoveryBanner } from "./NodeRecoveryBanner";
import { NetworkModeBanner } from "./NetworkModeBanner";
import { ShutdownProgressOverlay } from "./ShutdownProgressOverlay";
import { SyncStallBanner } from "./SyncStallBanner";
import { CoinSwitchOverlay } from "./CoinSwitchOverlay";
import { MobileHeader } from "./MobileHeader";
import { MobileTabBar } from "./MobileTabBar";
import { Sidebar } from "./Sidebar";
import { TopBar } from "./TopBar";

export function AppShell() {
  const coin = useActiveCoin();
  const { isLight, mobileOnly } = useWalletMode();

  if (mobileOnly) {
    return (
      <div className="mobile-shell flex min-h-screen w-full max-w-full flex-col overflow-x-hidden bg-bg text-fg">
        <ShutdownProgressOverlay />
        <NetworkModeBanner />
        <MobileHeader />
        <main className="mobile-main relative min-w-0 flex-1 overflow-y-auto overflow-x-hidden">
          <CoinSwitchOverlay />
          <div className="mx-auto flex w-full min-w-0 max-w-lg flex-col gap-4">
            <Outlet />
          </div>
        </main>
        <MobileTabBar />
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
