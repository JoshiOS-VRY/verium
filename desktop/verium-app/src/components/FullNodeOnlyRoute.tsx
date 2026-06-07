import { useEffect } from "react";
import { Navigate, Outlet, useLocation } from "react-router-dom";
import { useWalletMode } from "@/hooks/useWalletMode";
import { lightWalletCopy } from "@/lib/light-wallet/copy";
import { pushToast } from "@/lib/toast-store";

/** Redirects light-wallet users away from full-node-only pages. */
export function FullNodeOnlyRoute() {
  const { isLight, isLoading } = useWalletMode();
  const location = useLocation();

  useEffect(() => {
    if (!isLoading && isLight) {
      pushToast({
        title: "Full node required",
        description: lightWalletCopy.fullNodeOnlyToast,
        tone: "info",
      });
    }
  }, [isLight, isLoading, location.pathname]);

  if (isLoading) {
    return (
      <div className="flex min-h-48 items-center justify-center text-sm text-fg-muted">
        Loading…
      </div>
    );
  }

  if (isLight) {
    return <Navigate to="/dashboard" replace />;
  }

  return <Outlet />;
}
