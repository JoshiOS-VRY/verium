import { Loader2 } from "lucide-react";
import { useAppCoinSwitchTransition } from "@/hooks/useAppCoinSwitchTransition";
import { useCoinProfile } from "@/lib/coin/context";

/** Covers main content while core queries load after a coin switch. */
export function CoinSwitchOverlay() {
  const switching = useAppCoinSwitchTransition();
  const profile = useCoinProfile();

  if (!switching) return null;

  return (
    <div
      className="pointer-events-none absolute inset-0 z-30 flex flex-col items-center justify-center gap-2 rounded-md bg-bg/60 backdrop-blur-[1px]"
      role="status"
      aria-live="polite"
      aria-busy="true"
      aria-label={`Switching to ${profile.displayName}`}
    >
      <Loader2 className="h-7 w-7 animate-spin text-accent" aria-hidden />
      <span className="text-sm font-medium text-fg">
        Switching to {profile.displayName}…
      </span>
    </div>
  );
}
