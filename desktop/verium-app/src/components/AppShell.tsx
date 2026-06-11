import { Outlet } from 'react-router-dom';
import { useActiveCoin } from '@/lib/coin/context';
import { useWalletMode } from '@/hooks/useWalletMode';
import { DashboardNodeActivity } from './DashboardNodeActivity';
import { NodeRecoveryBanner } from './NodeRecoveryBanner';
import { NetworkModeBanner } from './NetworkModeBanner';
import { ShutdownProgressOverlay } from './ShutdownProgressOverlay';
import { SyncStallBanner } from './SyncStallBanner';
import { CoinSwitchOverlay } from './CoinSwitchOverlay';
import { Sidebar } from './Sidebar';
import { TopBar } from './TopBar';

export function AppShell() {
  const coin = useActiveCoin();
  const { isLight } = useWalletMode();

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-bg text-fg">
      <ShutdownProgressOverlay />
      <Sidebar />
      <div className="flex min-w-0 flex-1 flex-col">
        {/* Network mode banner: persistent at the very top when on
            binarytest. See vericoin/doc/dace/binarytest-network.md. */}
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
