import { Link, useLocation } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { AlertTriangle } from 'lucide-react';
import { useActiveCoin } from '@/lib/coin/context';
import { coinQueryKey } from '@/lib/coin/profile';
import { useWalletMode } from '@/hooks/useWalletMode';
import { lightWalletExists } from '@/lib/light-wallet/client';
import { lightWalletCopy } from '@/lib/light-wallet/copy';
import { rpcGetWalletInfo } from '@/lib/rpc/client';
import { walletInfoForMode } from '@/lib/wallet-unlock';

/** Shown in light mode when no encrypted keystore exists for the active chain. */
export function LightWalletMissingBanner() {
  const coin = useActiveCoin();
  const location = useLocation();
  const { isLight, mobileOnly } = useWalletMode();
  const exists = useQuery({
    queryKey: coinQueryKey(coin, 'light-wallet-exists'),
    queryFn: () => lightWalletExists(coin),
    enabled: isLight,
    staleTime: 0,
    refetchOnMount: 'always',
  });
  const wallet = useQuery({
    queryKey: coinQueryKey(coin, 'getwalletinfo'),
    queryFn: () => rpcGetWalletInfo(coin),
    enabled: isLight && exists.data !== false,
    refetchInterval: false,
  });
  const effectiveWallet = walletInfoForMode(true, wallet.data);

  if (!isLight || exists.isLoading || exists.isFetching || wallet.isLoading) {
    return null;
  }
  if (exists.data === true || effectiveWallet) return null;
  if (exists.data !== false) return null;

  if (mobileOnly) {
    return (
      <div className="mobile-banner border border-warning/40 bg-warning/10">
        <div className="flex items-start gap-2 font-medium text-fg">
          <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0 text-warning" />
          Set up this chain
        </div>
        <p className="mt-2 text-xs leading-relaxed text-fg-muted">
          Create or import a wallet for {coin === 'verium' ? 'Verium' : 'Vericoin'} to get started.
        </p>
        <Link
          to="/setup"
          state={{ lightWalletSetup: coin, from: location.pathname }}
          className="mt-3 inline-flex h-10 w-full items-center justify-center rounded-xl bg-accent text-sm font-semibold text-accent-fg"
        >
          {lightWalletCopy.dashboardNoWalletCta}
        </Link>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-2 rounded-md border border-warning/40 bg-warning/10 px-4 py-3 text-sm">
      <div className="flex items-start gap-2 font-medium text-fg">
        <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0 text-warning" />
        {lightWalletCopy.dashboardNoWalletTitle}
      </div>
      <p className="text-xs text-fg-muted">{lightWalletCopy.dashboardNoWalletBody}</p>
      <Link
        to="/setup"
        state={{ lightWalletSetup: coin, from: location.pathname }}
        className="text-xs font-medium text-accent underline"
      >
        {lightWalletCopy.dashboardNoWalletCta}
      </Link>
    </div>
  );
}
