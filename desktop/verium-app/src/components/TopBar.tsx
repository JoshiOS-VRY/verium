import { Link, useLocation } from "react-router-dom";
import { ArrowLeft } from "lucide-react";
import { APP_ROUTE_TITLES } from "@/lib/app-nav";
import { DaemonStatusBadge } from "./DaemonStatusBadge";
import { LightServerBadge } from "./LightServerBadge";
import { useWalletMode } from "@/hooks/useWalletMode";
import { cn } from "@/lib/utils";

export function TopBar() {
  const { pathname } = useLocation();
  const { isLight } = useWalletMode();
  const title = APP_ROUTE_TITLES[pathname] ?? "Vericonomy Wallet";
  return (
    <header className="flex h-14 shrink-0 items-center justify-between border-b border-border bg-bg-subtle px-8">
      <div className="flex min-w-0 items-center gap-4">
        <Link
          to="/setup"
          state={{ setupHub: true }}
          className={cn(
            "inline-flex shrink-0 items-center gap-2 rounded-md px-2.5 py-1.5 text-sm font-medium text-fg-muted",
            "transition-colors hover:bg-bg-panel hover:text-fg",
          )}
        >
          <ArrowLeft className="h-4 w-4" aria-hidden />
          Back to wallet menu
        </Link>
        <h1 className="truncate text-lg font-semibold">{title}</h1>
      </div>
      {isLight ? <LightServerBadge /> : <DaemonStatusBadge />}
    </header>
  );
}
