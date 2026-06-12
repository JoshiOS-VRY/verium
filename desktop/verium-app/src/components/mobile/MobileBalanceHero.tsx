import { useQuery } from '@tanstack/react-query';
import { Loader2 } from 'lucide-react';
import { useActiveCoin, useCoinProfile } from '@/lib/coin/context';
import { coinQueryKey } from '@/lib/coin/profile';
import { rpcGetWalletInfo } from '@/lib/rpc/client';
import { formatCoinAmount } from '@/lib/units';
import { cn } from '@/lib/utils';
import { lockedWalletBalanceClass } from '@/lib/wallet-unlock';

export function MobileBalanceHero() {
  const coin = useActiveCoin();
  const profile = useCoinProfile();
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

  const spendable = wallet.data.balance;
  const unconfirmed = wallet.data.unconfirmed_balance;
  const immature = wallet.data.immature_balance;
  const total = spendable + unconfirmed + immature;
  const blurClass = lockedWalletBalanceClass(wallet.data);
  const scanning = typeof wallet.data.scanning === 'object' ? wallet.data.scanning : null;

  return (
    <section className="mobile-balance-hero rounded-2xl border border-border bg-gradient-to-br from-bg-panel to-bg-subtle/60 p-5 shadow-sm">
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
      <p className="mt-1 text-center text-sm text-fg-muted">{profile.symbol}</p>

      <dl className="mt-5 grid grid-cols-3 gap-2 border-t border-border/60 pt-4 text-center">
        <div>
          <dt className="text-[10px] font-medium uppercase tracking-wide text-fg-subtle">
            Spendable
          </dt>
          <dd className={cn('mt-1 text-xs font-semibold tabular-nums text-fg', blurClass)}>
            {formatCoinAmount(spendable, coin, 2)}
          </dd>
        </div>
        <div>
          <dt className="text-[10px] font-medium uppercase tracking-wide text-fg-subtle">
            Pending
          </dt>
          <dd className={cn('mt-1 text-xs font-semibold tabular-nums text-fg', blurClass)}>
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

      {scanning && (
        <p className="mt-3 flex items-center justify-center gap-2 text-xs text-warning">
          <Loader2 className="h-3.5 w-3.5 animate-spin" />
          Scanning {Math.round((scanning.progress ?? 0) * 100)}%
        </p>
      )}
    </section>
  );
}
