import { lazy, Suspense, useEffect, useRef } from 'react';
import { Navigate, Route, Routes, useLocation, useNavigate } from 'react-router-dom';
import { AppShell } from '@/components/AppShell';
import { AppErrorBoundary } from '@/components/AppErrorBoundary';
import { CoinProvider } from '@/lib/coin/context';
import { useActiveCoin } from '@/lib/coin/context';
import type { CoinId } from '@/lib/coin/profile';
import { BINARYTEST_ENABLED } from '@/lib/features';
import { useUserPreferences } from '@/lib/user-preferences';
import { useWebAudioGestureUnlock } from '@/lib/web-audio';
import { useAutoLock } from '@/hooks/useAutoLock';
import { useScheduledBackup } from '@/hooks/useScheduledBackup';
import { useMiningPollCoordinator } from '@/hooks/useMiningPollCoordinator';
import { useAdaptiveMiningThreads } from '@/hooks/useAdaptiveMiningThreads';
import { useLightWalletBalanceSync } from '@/hooks/useLightWalletBalanceSync';
import { useWalletInfoPollCoordinator } from '@/hooks/useWalletInfoPollCoordinator';
import { useBlockchainInfoPollCoordinator } from '@/hooks/useBlockchainInfoPollCoordinator';
import { useExplorerStatsPollCoordinator } from '@/hooks/useExplorerStatsPollCoordinator';
import { useVericoinEarnPollCoordinator } from '@/hooks/useVericoinEarnPollCoordinator';
import { useAutoMine } from '@/hooks/useAutoMine';
import { useAutoStake } from '@/hooks/useAutoStake';
import { useBlockMinedSound } from '@/hooks/useBlockMinedSound';
import { useBlockMinedWatcher } from '@/hooks/useBlockMinedWatcher';
import { useBlockMinedDashboardSync } from '@/hooks/useBlockMinedDashboardSync';
import { useStakeRewardSound } from '@/hooks/useStakeRewardSound';
import { useStakeRewardWatcher } from '@/hooks/useStakeRewardWatcher';
import { useStakeRewardDashboardSync } from '@/hooks/useStakeRewardDashboardSync';
import { useChainTipWatcher } from '@/hooks/useChainTipWatcher';
import { useIncomingVrmNotifications } from '@/hooks/useIncomingVrmNotifications';
import { useIncomingVrmWatcher } from '@/hooks/useIncomingVrmWatcher';
import { useLightIncomingChainSync } from '@/hooks/useLightIncomingChainSync';
import { useIncomingVrcNotifications } from '@/hooks/useIncomingVrcNotifications';
import { useIncomingVrcWatcher } from '@/hooks/useIncomingVrcWatcher';
import { FullNodeOnlyRoute } from '@/components/FullNodeOnlyRoute';
import { useWalletProfile } from '@/hooks/useWalletProfile';
import { isProfileReady } from '@/lib/wallet-profile';
import { isProfileOpenable } from '@/lib/setup';
import { useTheme } from '@/hooks/useTheme';
import { useDeepLinkHandler } from '@/hooks/useDeepLinkHandler';
import { ToastHost } from '@/components/ToastHost';
const Setup = lazy(() => import('@/pages/Setup').then((m) => ({ default: m.Setup })));
const Dashboard = lazy(() => import('@/pages/Dashboard').then((m) => ({ default: m.Dashboard })));
const Mining = lazy(() => import('@/pages/Mining').then((m) => ({ default: m.Mining })));
const Staking = lazy(() => import('@/pages/Staking').then((m) => ({ default: m.Staking })));
const Network = lazy(() => import('@/pages/Network').then((m) => ({ default: m.Network })));
const Transactions = lazy(() =>
  import('@/pages/Transactions').then((m) => ({ default: m.Transactions }))
);
const Logs = lazy(() => import('@/pages/Logs').then((m) => ({ default: m.Logs })));
const RpcConsole = lazy(() =>
  import('@/pages/RpcConsole').then((m) => ({ default: m.RpcConsole }))
);
const Settings = lazy(() => import('@/pages/Settings').then((m) => ({ default: m.Settings })));
const Security = lazy(() => import('@/pages/Security').then((m) => ({ default: m.Security })));
const SignVerify = lazy(() =>
  import('@/pages/SignVerify').then((m) => ({ default: m.SignVerify }))
);
const Resources = lazy(() => import('@/pages/Resources').then((m) => ({ default: m.Resources })));
const AddressBook = lazy(() =>
  import('@/pages/AddressBook').then((m) => ({ default: m.AddressBook }))
);
const BinaryChain = lazy(() =>
  import('@/pages/BinaryChain').then((m) => ({ default: m.BinaryChain }))
);
const ExplorerTransactionPage = lazy(() =>
  import('@/pages/mobile/ExplorerTransactionPage').then((m) => ({
    default: m.ExplorerTransactionPage,
  }))
);
const ExplorerBlockPage = lazy(() =>
  import('@/pages/mobile/ExplorerBlockPage').then((m) => ({ default: m.ExplorerBlockPage }))
);
const ExplorerAddressPage = lazy(() =>
  import('@/pages/mobile/ExplorerAddressPage').then((m) => ({ default: m.ExplorerAddressPage }))
);

function RouteFallback() {
  return (
    <div className="flex min-h-48 items-center justify-center text-sm text-fg-muted">Loading…</div>
  );
}

function SetupRedirect() {
  const navigate = useNavigate();
  const location = useLocation();
  const coin = useActiveCoin();
  const loaded = useUserPreferences((s) => s.loaded);
  /** Sticky per-coin ready — avoids setup bounce while profile refetches on switch. */
  const confirmedReady = useRef<Set<CoinId>>(new Set());
  const { data: profile, isLoading: profileLoading } = useWalletProfile(coin, loaded);

  useEffect(() => {
    if (!loaded) return;
    if (profile && (isProfileReady(profile) || isProfileOpenable(profile))) {
      confirmedReady.current.add(coin);
    }
    if (profileLoading && confirmedReady.current.has(coin)) return;
    if (!profile) {
      if (confirmedReady.current.has(coin)) return;
      return;
    }
    if (isProfileReady(profile)) return;
    if (isProfileOpenable(profile)) return;
    // Full-node mode with only a light wallet: don't trap the user in setup.
    if (
      profile.mode === 'full_node' &&
      profile.keys_present.light &&
      !profile.keys_present.full_node
    ) {
      return;
    }
    if (location.pathname === '/setup') return;
    navigate('/setup', { replace: true, state: { setupHub: true } });
  }, [loaded, profileLoading, profile, coin, location.pathname, navigate]);

  return null;
}

function AppHooks() {
  const prefs = useUserPreferences((s) => s.prefs);
  useAutoMine();
  useMiningPollCoordinator();
  useLightWalletBalanceSync();
  useWalletInfoPollCoordinator();
  useBlockchainInfoPollCoordinator();
  useExplorerStatsPollCoordinator();
  useVericoinEarnPollCoordinator();
  useAdaptiveMiningThreads();
  useAutoStake();
  useAutoLock();
  useScheduledBackup();
  useTheme();
  useChainTipWatcher();
  useBlockMinedWatcher();
  useBlockMinedDashboardSync();
  useBlockMinedSound();
  useStakeRewardWatcher();
  useStakeRewardDashboardSync();
  useStakeRewardSound();
  useLightIncomingChainSync();
  useIncomingVrmWatcher();
  useIncomingVrmNotifications();
  useIncomingVrcWatcher();
  useIncomingVrcNotifications();
  useWebAudioGestureUnlock(
    prefs.play_sound_on_block_mined === true ||
      prefs.play_sound_on_stake_reward === true ||
      prefs.notify_on_vrm_received !== false ||
      prefs.notify_on_vrc_received !== false
  );
  return null;
}

function AppRoutes() {
  const load = useUserPreferences((s) => s.load);
  useDeepLinkHandler();

  useEffect(() => {
    void load();
  }, [load]);

  return (
    <>
      <AppHooks />
      <SetupRedirect />
      <ToastHost />
      {/* <MemoryDiagnosticsPanel /> */}
      <Suspense fallback={<RouteFallback />}>
        <Routes>
          <Route path="/setup" element={<Setup />} />
          <Route element={<AppShell />}>
            <Route path="/" element={<Navigate to="/dashboard" replace />} />
            <Route path="/dashboard" element={<Dashboard />} />
            <Route path="/wallet" element={<Navigate to="/dashboard" replace />} />
            <Route element={<FullNodeOnlyRoute />}>
              <Route path="/mining" element={<Mining />} />
              <Route path="/staking" element={<Staking />} />
              <Route path="/network" element={<Network />} />
              <Route path="/sign" element={<SignVerify />} />
              <Route path="/console" element={<RpcConsole />} />
              <Route path="/logs" element={<Logs />} />
            </Route>
            {BINARYTEST_ENABLED && <Route path="/binary-chain" element={<BinaryChain />} />}
            <Route path="/transactions" element={<Transactions />} />
            <Route path="/explorer/tx/:txid" element={<ExplorerTransactionPage />} />
            <Route path="/explorer/block/:id" element={<ExplorerBlockPage />} />
            <Route path="/explorer/address/:address" element={<ExplorerAddressPage />} />
            <Route path="/addresses" element={<AddressBook />} />
            <Route path="/resources" element={<Resources />} />
            <Route path="/settings" element={<Settings />} />
            <Route path="/security" element={<Security />} />
          </Route>
          <Route path="*" element={<Navigate to="/dashboard" replace />} />
        </Routes>
      </Suspense>
    </>
  );
}

export default function App() {
  return (
    <AppErrorBoundary>
      <CoinProvider>
        <AppRoutes />
      </CoinProvider>
    </AppErrorBoundary>
  );
}
