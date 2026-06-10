import { NavLink } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { Lock } from "lucide-react";
import { CoinSwitcher } from "@/components/CoinSwitcher";
import { QuitWalletButton } from "@/components/QuitWalletButton";
import { useActiveCoin, useEnabledCoins } from "@/lib/coin/context";
import { coinQueryKey } from "@/lib/coin/profile";
import { APP_NAV_ITEMS, filterAppNavItems } from "@/lib/app-nav";
import { BINARYTEST_ENABLED } from "@/lib/features";
import { useIsTestNetwork } from "@/lib/network-mode";
import { rpcGetWalletInfo } from "@/lib/rpc/client";
import { useWalletMode } from "@/hooks/useWalletMode";
import { cn } from "@/lib/utils";
import { isWalletLocked } from "@/lib/wallet-unlock";

const APP_VERSION =
  (import.meta as unknown as { env: Record<string, string> }).env
    ?.VITE_APP_VERSION || "1.0.0";

export function Sidebar() {
  const activeCoin = useActiveCoin();
  const enabledCoins = useEnabledCoins();
  const isTestNetwork = useIsTestNetwork();
  const { isLight } = useWalletMode();

  const wallet = useQuery({
    queryKey: coinQueryKey(activeCoin, "getwalletinfo"),
    queryFn: () => rpcGetWalletInfo(activeCoin),
    refetchInterval: false,
  });
  const walletLocked = isWalletLocked(wallet.data);

  const visibleItems = filterAppNavItems(APP_NAV_ITEMS, {
    activeCoin,
    enabledCoins,
    isLight,
    isTestNetwork,
    binarytestEnabled: BINARYTEST_ENABLED,
  });

  return (
    <aside className="flex h-full w-60 shrink-0 flex-col border-r border-border bg-bg-subtle">
      <div className="px-3 py-4">
        <CoinSwitcher />
      </div>

      <nav className="flex flex-1 flex-col gap-0.5 px-2 py-2">
        {visibleItems.map(({ to, label, icon: Icon, requiresPassphrase }) => {
          const showLock = Boolean(requiresPassphrase) && walletLocked;
          return (
            <NavLink
              key={to}
              to={to}
              className={({ isActive }) =>
                cn(
                  "flex items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors",
                  isActive
                    ? "bg-bg-panel text-fg"
                    : "text-fg-muted hover:bg-bg-panel hover:text-fg",
                )
              }
            >
              <Icon className="h-4 w-4" />
              <span className="flex-1 truncate">{label}</span>
              {showLock && (
                <Lock
                  className="h-3.5 w-3.5 shrink-0 text-amber-400"
                  aria-label="Locked — passphrase required"
                />
              )}
            </NavLink>
          );
        })}
      </nav>

      <div className="border-t border-border px-5 py-3 text-xs text-fg-subtle">
        <QuitWalletButton />
        Vericonomy Wallet v{APP_VERSION}
        <div className="mt-0.5 flex items-center gap-2 text-[10px] uppercase tracking-wider">
          <NetworkBadge />
        </div>
      </div>
    </aside>
  );
}

function NetworkBadge() {
  const isTest = useIsTestNetwork();
  if (!BINARYTEST_ENABLED || !isTest) return null;
  return (
    <span className="rounded border border-amber-500/40 bg-amber-500/20 px-1.5 py-[1px] text-[9px] font-semibold normal-case text-amber-200">
      binarytest
    </span>
  );
}
