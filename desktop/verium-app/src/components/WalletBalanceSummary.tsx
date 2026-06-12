import { useQuery } from '@tanstack/react-query';
import { Loader2 } from 'lucide-react';
import { useActiveCoin } from '@/lib/coin/context';
import { coinQueryKey } from '@/lib/coin/profile';
import { rpcGetWalletInfo } from '@/lib/rpc/client';
import { formatCoinAmount } from '@/lib/units';
import { coinMaturityConfirmations } from '@/lib/units';
import { cn } from '@/lib/utils';
import { lockedWalletBalanceClass } from '@/lib/wallet-unlock';

export function WalletBalanceSummary() {
  const coin = useActiveCoin();
  const wallet = useQuery({
    queryKey: coinQueryKey(coin, 'getwalletinfo'),
    queryFn: () => rpcGetWalletInfo(coin),
    refetchInterval: false,
  });

  if (wallet.isLoading || !wallet.data) return null;

  const isLight = wallet.data.light_wallet === true;
  const available = wallet.data.balance;
  const confirmed = wallet.data.confirmed_balance ?? available;
  const unconfirmed = wallet.data.unconfirmed_balance;
  const immature = wallet.data.immature_balance;
  const total = isLight ? available + immature : available + unconfirmed + immature;
  const scanning = typeof wallet.data.scanning === 'object' ? wallet.data.scanning : null;
  const mature = coinMaturityConfirmations(coin);
  const blurClass = lockedWalletBalanceClass(wallet.data);

  return (
    <div className="rounded-md border border-border bg-bg-subtle px-4 py-3 text-sm">
      <div className="flex flex-wrap items-baseline justify-between gap-2">
        <div>
          <span className="text-fg-muted">Wallet total </span>
          <span className={cn('text-lg font-semibold tabular-nums', blurClass)}>
            {formatCoinAmount(total, coin, 4)}
          </span>
        </div>
        <div className="flex flex-wrap gap-x-4 gap-y-1 text-xs text-fg-muted">
          <span>
            {isLight ? 'Available' : 'Spendable'}{' '}
            <span className={cn('font-medium tabular-nums text-fg', blurClass)}>
              {formatCoinAmount(available, coin, 4)}
            </span>
          </span>
          {isLight && (
            <span>
              Confirmed{' '}
              <span className={cn('font-medium tabular-nums text-fg', blurClass)}>
                {formatCoinAmount(confirmed, coin, 4)}
              </span>
            </span>
          )}
          <span>
            {isLight ? 'Pending change' : 'Unconfirmed'}{' '}
            <span className={cn('font-medium tabular-nums text-fg', blurClass)}>
              {formatCoinAmount(unconfirmed, coin, 4)}
            </span>
          </span>
          <span>
            Immature{' '}
            <span className={cn('font-medium tabular-nums text-fg', blurClass)}>
              {formatCoinAmount(immature, coin, 4)}
            </span>
          </span>
        </div>
      </div>
      {scanning && (
        <div className="mt-2 flex items-center gap-2 text-xs text-warning">
          <Loader2 className="h-3.5 w-3.5 animate-spin" />
          Rescanning wallet… {Math.round((scanning.progress ?? 0) * 100)}% complete
        </div>
      )}
      <p className="mt-2 text-xs text-fg-subtle">
        Mined or staked rewards stay immature until {mature} confirmations.
      </p>
    </div>
  );
}
