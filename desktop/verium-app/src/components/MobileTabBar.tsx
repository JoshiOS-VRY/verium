import { NavLink } from "react-router-dom";
import { useActiveCoin, useEnabledCoins } from "@/lib/coin/context";
import { BINARYTEST_ENABLED } from "@/lib/features";
import { useIsTestNetwork } from "@/lib/network-mode";
import { APP_NAV_ITEMS, filterAppNavItems } from "@/lib/app-nav";
import { useWalletMode } from "@/hooks/useWalletMode";
import { cn } from "@/lib/utils";

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
      className="mobile-tab-bar fixed inset-x-0 bottom-0 z-40 border-t border-border bg-bg-subtle/95 backdrop-blur-md"
      aria-label="Primary"
    >
      <div className="mx-auto flex max-w-lg items-stretch justify-around px-1 pt-1">
        {tabs.map(({ to, shortLabel, label, icon: Icon }) => (
          <NavLink
            key={to}
            to={to}
            className={({ isActive }) =>
              cn(
                "flex min-w-0 flex-1 flex-col items-center gap-0.5 rounded-lg px-2 py-2 text-[10px] font-medium transition-colors",
                isActive
                  ? "text-accent"
                  : "text-fg-muted hover:text-fg",
              )
            }
          >
            <Icon className="h-5 w-5 shrink-0" aria-hidden />
            <span className="truncate">{shortLabel ?? label}</span>
          </NavLink>
        ))}
      </div>
    </nav>
  );
}
