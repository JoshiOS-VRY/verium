import { NavLink } from 'react-router-dom';
import { useActiveCoin, useEnabledCoins } from '@/lib/coin/context';
import { BINARYTEST_ENABLED } from '@/lib/features';
import { useIsTestNetwork } from '@/lib/network-mode';
import { APP_NAV_ITEMS, filterAppNavItems } from '@/lib/app-nav';
import { useWalletMode } from '@/hooks/useWalletMode';
import { cn } from '@/lib/utils';

export function MobileTabBar() {
  const activeCoin = useActiveCoin();
  const enabledCoins = useEnabledCoins();
  const isTestNetwork = useIsTestNetwork();
  const { isLight } = useWalletMode();

  const tabs = filterAppNavItems(APP_NAV_ITEMS, {
    activeCoin,
    enabledCoins,
    isLight,
    isTestNetwork,
    binarytestEnabled: BINARYTEST_ENABLED,
  }).filter((item) => item.mobileTab);

  return (
    <nav
      className="mobile-tab-bar shrink-0 border-t border-border bg-bg-subtle/95 backdrop-blur-md"
      aria-label="Primary"
    >
      <div className="mobile-tab-bar-inner mx-auto flex max-w-lg items-stretch justify-around">
        {tabs.map(({ to, shortLabel, label, icon: Icon }) => (
          <NavLink
            key={to}
            to={to}
            className={({ isActive }) =>
              cn(
                'mobile-tab-bar-item flex min-w-0 flex-1 flex-col items-center justify-center gap-0.5 rounded-lg px-2 text-[11px] font-medium leading-tight transition-colors',
                isActive ? 'text-accent' : 'text-fg-muted hover:text-fg'
              )
            }
          >
            <Icon className="h-[22px] w-[22px] shrink-0" aria-hidden />
            <span className="truncate max-w-full">{shortLabel ?? label}</span>
          </NavLink>
        ))}
      </div>
    </nav>
  );
}
