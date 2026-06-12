import { useQuery } from '@tanstack/react-query';
import { Loader2 } from 'lucide-react';
import { useActiveCoin } from '@/lib/coin/context';
import { coinQueryKey } from '@/lib/coin/profile';
import { useWalletMode } from '@/hooks/useWalletMode';
import { lightWalletCopy } from '@/lib/light-wallet/copy';
import { rpcGetWalletInfo } from '@/lib/rpc/client';

/** Shown while the wallet is performing its initial Electrum gap scan after unlock. */
export function LightWalletSyncBanner() {
  const coin = useActiveCoin();
  const { isLight, mobileOnly } = useWalletMode();
  const wallet = useQuery({
    queryKey: coinQueryKey(coin, 'getwalletinfo'),
    queryFn: () => rpcGetWalletInfo(coin),
    enabled: isLight,
  });

  if (!isLight || wallet.data?.light_syncing !== true) return null;

  const balanceReady =
    (wallet.data?.balance ?? 0) > 0 && wallet.data?.private_keys_enabled === true;

  const message = balanceReady
    ? mobileOnly
      ? 'Balance ready — background scan still running.'
      : 'Address scan still running in the background. Confirmed balance is available — you can send now.'
    : lightWalletCopy.unlockRefreshingBalance;

  return (
    <div
      className={
        mobileOnly
          ? 'mobile-banner flex items-start gap-2 border border-accent/30 bg-accent/5 text-fg-muted'
          : 'flex items-start gap-2 rounded-md border border-accent/30 bg-accent/5 px-3 py-2 text-xs text-fg-muted'
      }
    >
      <Loader2 className="mt-0.5 h-3.5 w-3.5 shrink-0 animate-spin text-accent" />
      <span className={mobileOnly ? 'text-xs leading-relaxed' : undefined}>{message}</span>
    </div>
  );
}
