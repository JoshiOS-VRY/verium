import { useActiveCoin, useEnabledCoins } from '@/lib/coin/context';
import { BINARYTEST_ENABLED } from '@/lib/features';
import { useIsTestNetwork } from '@/lib/network-mode';
import { APP_NAV_ITEMS, filterAppNavItems } from '@/lib/app-nav';
import { useWalletMode } from '@/hooks/useWalletMode';

/** Primary navigation tabs for the mobile / light-wallet shell. */
export function useMobileNavTabs() {
  const activeCoin = useActiveCoin();
  const enabledCoins = useEnabledCoins();
  const isTestNetwork = useIsTestNetwork();
  const { isLight } = useWalletMode();

  return filterAppNavItems(APP_NAV_ITEMS, {
    activeCoin,
    enabledCoins,
    isLight,
    isTestNetwork,
    binarytestEnabled: BINARYTEST_ENABLED,
  }).filter((item) => item.mobileTab);
}
