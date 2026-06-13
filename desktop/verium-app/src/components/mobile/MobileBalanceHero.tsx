import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Loader2 } from 'lucide-react';
import { useActiveCoin, useCoinProfile } from '@/lib/coin/context';
import { coinQueryKey } from '@/lib/coin/profile';
import { rpcGetWalletInfo } from '@/lib/rpc/client';
import { formatCoinAmount } from '@/lib/units';
import { cn } from '@/lib/utils';
import { lockedWalletBalanceClass } from '@/lib/wallet-unlock';
import { useWalletMode } from '@/hooks/useWalletMode';
import { LightWalletSyncStatus } from '@/components/mobile/LightWalletSyncStatus';
import { WalletCumulativeBalanceChart } from '@/components/mobile/WalletCumulativeBalanceChart';

export function MobileBalanceHero({
  showChart = false,
  refreshing = false,
}: {
  showChart?: boolean;
  refreshing?: boolean;
}) {
  const coin = useActiveCoin();
  const profile = useCoinProfile();
  const { isLight } = useWalletMode();
  const wallet = useQuery({
    queryKey: coinQueryKey(coin, 'getwalletinfo'),
    queryFn: () => rpcGetWalletInfo(coin),
    refetchInterval: false,
  });

  if (wallet.isLoading) {
    return (
      <div className="mobile-balance-hero flex items-center justify-center py-10">
        <Loader2 className="h-6 w-6 animate-spin text-accent" />
      </div>
    );
  }

  if (!wallet.data) return null;

  const available = wallet.data.balance;
  const confirmed = wallet.data.confirmed_balance ?? available;
  const unconfirmed = wallet.data.unconfirmed_balance;
  const immature = wallet.data.immature_balance;
  const total = isLight ? available + immature : available + unconfirmed + immature;
  const blurClass = lockedWalletBalanceClass(wallet.data);
  const lightSyncing = wallet.data.light_syncing === true;
  const unlocked = wallet.data.private_keys_enabled === true;
  const hasPending = unconfirmed > 0 || immature > 0;

  return (
    <section className="mobile-balance-hero rounded-2xl border border-border bg-gradient-to-br from-bg-panel to-bg-subtle/60 p-5 shadow-sm">
      <LightWalletSyncStatus
        wallet={wallet.data}
        unlocked={unlocked}
        isLight={isLight}
        refreshing={refreshing}
        className="mb-2"
      />

      <p className="text-center text-xs font-medium uppercase tracking-wide text-fg-subtle">
        {profile.displayName} balance
      </p>
      <p
        className={cn(
          'mt-2 text-center text-3xl font-bold tabular-nums tracking-tight text-fg',
          blurClass
        )}
      >
        {formatCoinAmount(total, coin, 4)}
      </p>

      {showChart && <WalletCumulativeBalanceChart embedded blurClass={blurClass} />}

      <dl className="mt-5 grid grid-cols-3 gap-2 border-t border-border/60 pt-4 text-center">
        <div>
          <dt className="text-[10px] font-medium uppercase tracking-wide text-fg-subtle">
            {isLight ? 'Confirmed' : 'Spendable'}
          </dt>
          <dd className={cn('mt-1 text-xs font-semibold tabular-nums text-fg', blurClass)}>
            {formatCoinAmount(isLight ? confirmed : available, coin, 2)}
          </dd>
        </div>
        <div>
          <dt
            className={cn(
              'text-[10px] font-medium uppercase tracking-wide',
              hasPending ? 'text-amber-600 dark:text-amber-400' : 'text-fg-subtle'
            )}
          >
            {isLight ? 'Pending change' : 'Pending'}
          </dt>
          <dd
            className={cn(
              'mt-1 text-xs font-semibold tabular-nums',
              hasPending ? 'text-amber-700 dark:text-amber-300' : 'text-fg',
              blurClass
            )}
          >
            {formatCoinAmount(unconfirmed, coin, 2)}
          </dd>
        </div>
        <div>
          <dt className="text-[10px] font-medium uppercase tracking-wide text-fg-subtle">
            Immature
          </dt>
          <dd className={cn('mt-1 text-xs font-semibold tabular-nums text-fg', blurClass)}>
            {formatCoinAmount(immature, coin, 2)}
          </dd>
        </div>
      </dl>

      {lightSyncing && wallet.data.light_scan_progress == null && (
        <p className="mt-3 flex items-center justify-center gap-2 text-xs text-warning">
          <Loader2 className="h-3.5 w-3.5 animate-spin" />
          Scanning addresses for indexed balance…
        </p>
      )}
    </section>
  );
}

/** Hook for dashboard pull-to-refresh local refreshing state. */
export function useMobileBalanceRefreshing() {
  const [refreshing, setRefreshing] = useState(false);
  return {
    refreshing,
    async runRefresh(fn: () => Promise<void>) {
      setRefreshing(true);
      try {
        await fn();
      } finally {
        setRefreshing(false);
      }
    },
  };
}
