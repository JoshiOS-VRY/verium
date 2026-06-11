import { Link, useLocation } from "react-router-dom";
import { ArrowLeft } from "lucide-react";
import { CoinSwitcher } from "@/components/CoinSwitcher";
import { DaemonStatusBadge } from "@/components/DaemonStatusBadge";
import { LightServerBadge } from "@/components/LightServerBadge";
import { APP_ROUTE_TITLES } from "@/lib/app-nav";
import { useWalletMode } from "@/hooks/useWalletMode";
import { cn } from "@/lib/utils";

export function MobileHeader() {
  const { pathname } = useLocation();
  const { isLight } = useWalletMode();
  const title = APP_ROUTE_TITLES[pathname] ?? "Vericonomy Wallet";

  return (
    <header className="mobile-header sticky top-0 z-30 shrink-0 border-b border-border bg-bg-subtle/95 backdrop-blur-md">
      <div className="flex min-w-0 items-center gap-2 px-3 py-2">
        <Link
          to="/setup"
          state={{ setupHub: true }}
          className={cn(
            "inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-lg text-fg-muted",
            "transition-colors hover:bg-bg-panel hover:text-fg",
          )}
          aria-label="Back to wallet menu"
        >
          <ArrowLeft className="h-4 w-4" />
        </Link>
        <div className="min-w-0 flex-1">
          <h1 className="truncate text-base font-semibold leading-tight">
            {title}
          </h1>
        </div>
        <div className="min-w-0 max-w-[42%]">
          {isLight ? <LightServerBadge /> : <DaemonStatusBadge />}
        </div>
      </div>
      <div className="min-w-0 border-t border-border/60 px-3 py-2">
        <CoinSwitcher />
      </div>
    </header>
  );
}
