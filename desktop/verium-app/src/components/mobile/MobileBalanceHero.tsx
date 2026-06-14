import { useCallback, useMemo, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Loader2 } from 'lucide-react';
import { AnimatedNumber } from '@/components/AnimatedNumber';
import { useActiveCoin, useCoinProfile } from '@/lib/coin/context';
import { coinQueryKey } from '@/lib/coin/profile';
import {
  balanceDayOverDayChange,
  formatChartPointDate,
  type ChartCumulativePoint,
} from '@/lib/cumulative-balance';
import { rpcGetWalletInfo } from '@/lib/rpc/client';
import { formatCoinAmount } from '@/lib/units';
import { cn } from '@/lib/utils';
import { lockedWalletBalanceClass } from '@/lib/wallet-unlock';
import { useWalletMode } from '@/hooks/useWalletMode';
import { LightWalletSyncStatus } from '@/components/mobile/LightWalletSyncStatus';
import { WalletCumulativeBalanceChart } from '@/components/mobile/WalletCumulativeBalanceChart';

function BalanceDayChange({
  points,
  currentBalance,
  referenceUnixSeconds,
  coin,
  className,
}: {
  points: ChartCumulativePoint[];
  currentBalance: number;
  referenceUnixSeconds: number;
  coin: ReturnType<typeof useActiveCoin>;
  className?: string;
}) {
  const change = useMemo(
    () => balanceDayOverDayChange(points, currentBalance, referenceUnixSeconds),
    [points, currentBalance, referenceUnixSeconds]
  );

  if (!change) return null;

  const { delta, tone } = change;
  const sign = tone === 'up' ? '+' : tone === 'down' ? '' : '';
  const label = `${sign}${formatCoinAmount(delta, coin, 1)}`;

  return (
    <p
      className={cn(
        'text-[10px] font-semibold tabular-nums leading-none',
        tone === 'up' && 'text-emerald-600 dark:text-emerald-400',
        tone === 'down' && 'text-red-600 dark:text-red-400',
        tone === 'flat' && 'text-fg-muted',
        className
      )}
      title="Change vs. 24 hours ago"
    >
      {label}
    </p>
  );
}

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

  const [chartPoints, setChartPoints] = useState<ChartCumulativePoint[]>([]);
  const [scrubPoint, setScrubPoint] = useState<ChartCumulativePoint | null>(null);
  const [scrubbing, setScrubbing] = useState(false);

  const handleScrubChange = useCallback((point: ChartCumulativePoint | null, active: boolean) => {
    setScrubbing(active);
    setScrubPoint(active ? point : null);
  }, []);

  const handlePointsChange = useCallback((points: ChartCumulativePoint[]) => {
    setChartPoints(points);
  }, []);

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
  const total = isLight
    ? (wallet.data.wallet_total ?? available + unconfirmed + immature)
    : available + unconfirmed + immature;
  const blurClass = lockedWalletBalanceClass(wallet.data);
  const unlocked = wallet.data.private_keys_enabled === true;
  const hasPending = unconfirmed > 0 || immature > 0;

  const displayBalance = scrubbing && scrubPoint ? scrubPoint.balance : total;
  const referenceTime =
    scrubbing && scrubPoint
      ? scrubPoint.time
      : chartPoints.length > 0
        ? chartPoints[chartPoints.length - 1].time
        : Math.floor(Date.now() / 1000);

  return (
    <section className="mobile-balance-hero rounded-2xl border border-border bg-gradient-to-br from-bg-panel to-bg-subtle/60 p-5 shadow-sm">
      <LightWalletSyncStatus
        wallet={wallet.data}
        unlocked={unlocked}
        isLight={isLight}
        refreshing={refreshing}
        className="mb-2"
      />

      <div className="relative min-h-[4.5rem]">
        {showChart && chartPoints.length > 0 && (
          <BalanceDayChange
            points={chartPoints}
            currentBalance={displayBalance}
            referenceUnixSeconds={referenceTime}
            coin={coin}
            className="absolute right-0 top-0"
          />
        )}

        <p className="text-center text-xs font-medium uppercase tracking-wide text-fg-subtle">
          {scrubbing ? 'Balance at point' : `${profile.displayName} balance`}
        </p>
        <p
          className={cn(
            'mt-2 text-center text-3xl font-bold tabular-nums tracking-tight text-fg',
            blurClass
          )}
        >
          <AnimatedNumber
            value={displayBalance}
            fractionDigits={4}
            showTrendColor={!scrubbing}
            format={(value, digits) => formatCoinAmount(value, coin, digits)}
            tension={scrubbing ? 220 : 120}
            friction={scrubbing ? 26 : 14}
          />
        </p>
        {scrubbing && scrubPoint && (
          <p className="mt-1 text-center text-[11px] text-fg-muted">
            {formatChartPointDate(scrubPoint.time, true)}
          </p>
        )}
      </div>

      {showChart && (
        <WalletCumulativeBalanceChart
          embedded
          blurClass={blurClass}
          onScrubChange={handleScrubChange}
          onPointsChange={handlePointsChange}
        />
      )}

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
