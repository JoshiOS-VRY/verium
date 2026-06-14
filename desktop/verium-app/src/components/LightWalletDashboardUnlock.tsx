import { useQuery } from '@tanstack/react-query';
import { AlertTriangle, Lock } from 'lucide-react';
import { Link } from 'react-router-dom';
import { Card, CardContent } from '@/components/ui/Card';
import { WalletUnlockForm } from '@/components/WalletUnlockForm';
import { useActiveCoin } from '@/lib/coin/context';
import { coinQueryKey } from '@/lib/coin/profile';
import { useResponsiveLayout } from '@/hooks/useResponsiveLayout';
import { lightWalletExists } from '@/lib/light-wallet/client';
import { needsLightWalletRecovery } from '@/lib/setup';
import { rpcGetWalletInfo } from '@/lib/rpc/client';
import { tauriWalletProfile } from '@/lib/wallet-profile';
import { isWalletLocked } from '@/lib/wallet-unlock';

/** Prompt to unlock the light wallet so balance and history can load from Electrum. */
export function LightWalletDashboardUnlock() {
  const coin = useActiveCoin();
  const { isLight, isPhoneLayout } = useResponsiveLayout();
  const exists = useQuery({
    queryKey: coinQueryKey(coin, 'light-wallet-exists'),
    queryFn: () => lightWalletExists(coin),
    enabled: isLight,
  });
  const profile = useQuery({
    queryKey: coinQueryKey(coin, 'wallet-profile'),
    queryFn: () => tauriWalletProfile(coin),
    enabled: isLight && exists.data === true,
  });
  const wallet = useQuery({
    queryKey: coinQueryKey(coin, 'getwalletinfo'),
    queryFn: () => rpcGetWalletInfo(coin),
    enabled: isLight && exists.data === true,
    refetchInterval: false,
  });

  if (!isLight || exists.data !== true) return null;
  if (profile.isLoading || wallet.isLoading) return null;

  if (needsLightWalletRecovery(profile.data, 'light')) {
    if (isPhoneLayout) {
      return (
        <div className="mobile-banner border border-warning/40 bg-warning/10">
          <div className="flex items-start gap-2 font-medium text-fg">
            <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0 text-warning" />
            Import recovery phrase
          </div>
          <p className="mt-2 text-xs leading-relaxed text-fg-muted">
            Wallet info is on this device but keys need to be restored from your recovery phrase.
          </p>
          <Link
            to="/setup"
            state={{ setupHub: false }}
            className="mt-3 inline-flex h-10 w-full items-center justify-center rounded-xl bg-accent text-sm font-semibold text-accent-fg"
          >
            Import phrase
          </Link>
        </div>
      );
    }
    return (
      <Card className="border-warning/40 bg-warning/10">
        <CardContent className="flex flex-col gap-3 py-4">
          <div className="flex items-center gap-2 text-sm font-medium text-fg">
            <AlertTriangle className="h-4 w-4 text-warning" />
            Import recovery phrase to access this wallet
          </div>
          <p className="text-xs text-fg-muted">
            Wallet metadata is on this device but saved keys must be restored from your recovery
            phrase. Import below — no Windows Credential Manager step required.
          </p>
          <Link
            to="/setup"
            state={{ setupHub: false }}
            className="text-sm font-medium text-accent underline"
          >
            Open Setup → Import recovery phrase
          </Link>
        </CardContent>
      </Card>
    );
  }

  const locked = !wallet.data || isWalletLocked(wallet.data);
  if (!locked) return null;

  if (isPhoneLayout) {
    return (
      <section className="mobile-panel overflow-hidden rounded-2xl border border-accent/30 bg-accent/5 p-4 shadow-sm">
        <div className="flex items-center gap-2 text-sm font-semibold text-fg">
          <Lock className="h-4 w-4 text-accent" />
          Unlock wallet
        </div>
        <p className="mt-2 text-xs leading-relaxed text-fg-muted">
          Enter your passphrase to send and view full transaction history.
        </p>
        <WalletUnlockForm
          title="Unlock light wallet"
          description="Wallet passphrase"
          className="mt-3 gap-3"
          autoBiometricUnlock
        />
      </section>
    );
  }

  return (
    <Card className="border-accent/30 bg-accent/5">
      <CardContent className="flex flex-col gap-3 py-4">
        <div className="flex items-center gap-2 text-sm font-medium text-fg">
          <Lock className="h-4 w-4 text-accent" />
          Unlock your light wallet to send and view transactions
        </div>
        <p className="text-xs text-fg-muted">
          Your keys stay on this device. Unlocking is instant; balance may already be visible. A
          background scan on Vericonomy servers refreshes history after you unlock.
        </p>
        <WalletUnlockForm
          title="Unlock light wallet"
          description="Enter your wallet passphrase."
          className="gap-3"
        />
      </CardContent>
    </Card>
  );
}
