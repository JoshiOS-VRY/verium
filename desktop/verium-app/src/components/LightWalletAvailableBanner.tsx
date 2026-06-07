import { Link } from "react-router-dom";
import { Cloud } from "lucide-react";
import { useWalletMode } from "@/hooks/useWalletMode";

/** Shown when light wallet is available but the user is still on full-node mode. */
export function LightWalletAvailableBanner() {
  const { lightWalletEnabled, isLight } = useWalletMode();

  if (!lightWalletEnabled || isLight) {
    return null;
  }

  return (
    <div className="flex flex-wrap items-center justify-between gap-2 rounded-md border border-accent/30 bg-accent/5 px-4 py-2.5 text-sm">
      <span className="inline-flex items-center gap-2 text-fg">
        <Cloud className="h-4 w-4 text-accent" />
        Light wallet is enabled. Switch in Settings to connect via Electrum
        instead of running a full node.
      </span>
      <Link
        to="/settings"
        className="shrink-0 rounded-md bg-accent px-3 py-1.5 text-xs font-medium text-accent-fg hover:bg-accent/90"
      >
        Open Settings → Wallet mode
      </Link>
    </div>
  );
}
