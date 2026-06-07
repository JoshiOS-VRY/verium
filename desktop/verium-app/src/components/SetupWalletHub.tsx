import { ArrowRight, CheckCircle2, Circle, Loader2 } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import {
  ALL_COINS,
  COIN_LOGO_URLS,
  COIN_PROFILES,
  coinQueryKey,
  type CoinId,
} from "@/lib/coin/profile";
import { useEnabledCoins } from "@/lib/coin/context";
import { useUserPreferences } from "@/lib/user-preferences";
import { lightWalletExists } from "@/lib/light-wallet/client";
import { lightWalletCopy } from "@/lib/light-wallet/copy";
import { tauriWalletFileStatus } from "@/lib/rpc/client";
import {
  fullNodeWalletExists,
  isCoinWalletReady,
  type WalletModeChoice,
} from "@/lib/setup";
import { LIGHT_WALLET_ENABLED } from "@/lib/features";
import { ThemeSegmented } from "@/components/ThemeSegmented";
import { Button } from "@/components/ui/Button";
import { useTheme } from "@/hooks/useTheme";
import { cn } from "@/lib/utils";
import type { UserPreferences } from "@/lib/user-preferences";

interface SetupWalletHubProps {
  walletMode: WalletModeChoice;
  onWalletModeChange: (mode: WalletModeChoice) => void;
  showAdvancedLightMode: boolean;
  onShowAdvancedLightMode: () => void;
  onSelectCoin: (coin: CoinId) => void | Promise<void>;
  onOpenDashboard: () => void;
  openingCoin?: CoinId | null;
}

function CoinHubCard({
  coin,
  prefs,
  walletMode,
  onSelect,
  opening,
}: {
  coin: CoinId;
  prefs: UserPreferences;
  walletMode: WalletModeChoice;
  onSelect: (coin: CoinId) => void | Promise<void>;
  opening: boolean;
}) {
  const profile = COIN_PROFILES[coin];
  const light = useQuery({
    queryKey: coinQueryKey(coin, "light-wallet-exists"),
    queryFn: () => lightWalletExists(coin),
  });
  const walletFile = useQuery({
    queryKey: coinQueryKey(coin, "wallet-file-status"),
    queryFn: () => tauriWalletFileStatus(coin),
    enabled: walletMode === "full_node",
  });
  const ready = isCoinWalletReady(coin, prefs, {
    walletMode,
    hasLightWallet: light.data,
    hasFullNodeWallet: fullNodeWalletExists(walletFile.data),
  });

  return (
    <button
      type="button"
      disabled={opening}
      onClick={() => void onSelect(coin)}
      className="flex flex-col gap-3 rounded-lg border border-border bg-bg-subtle p-4 text-left transition-colors hover:border-accent disabled:cursor-wait disabled:opacity-70"
    >
      <div className="flex items-start gap-3">
        <img
          src={COIN_LOGO_URLS[coin]}
          alt=""
          className="h-10 w-10 shrink-0 rounded-lg object-contain"
        />
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <span className="font-semibold text-fg">{profile.displayName}</span>
            <span
              className={cn(
                "rounded border px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide",
                profile.accentClass,
              )}
            >
              {profile.symbol}
            </span>
          </div>
          <p className="mt-0.5 text-xs text-fg-subtle">{profile.tagline}</p>
        </div>
      </div>
      <div className="flex items-center justify-between gap-2 text-xs">
        <span
          className={cn(
            "flex items-center gap-1.5",
            ready ? "text-success" : "text-fg-muted",
          )}
        >
          {ready ? (
            <CheckCircle2 className="h-3.5 w-3.5" />
          ) : (
            <Circle className="h-3.5 w-3.5" />
          )}
          {ready ? "Ready" : "Not set up"}
        </span>
        <span className="flex items-center gap-1 font-medium text-accent">
          {opening ? (
            <>
              Opening…
              <Loader2 className="h-3.5 w-3.5 animate-spin" />
            </>
          ) : (
            <>
              {ready ? "Open" : "Set up"}
              <ArrowRight className="h-3.5 w-3.5" />
            </>
          )}
        </span>
      </div>
    </button>
  );
}

export function SetupWalletHub({
  walletMode,
  onWalletModeChange,
  showAdvancedLightMode,
  onShowAdvancedLightMode,
  onSelectCoin,
  onOpenDashboard,
  openingCoin = null,
}: SetupWalletHubProps) {
  const enabledCoins = useEnabledCoins();
  const prefs = useUserPreferences((s) => s.prefs);
  const { mode: themeMode, setMode: setThemeMode } = useTheme();
  const options = ALL_COINS.filter((coin) => enabledCoins.includes(coin));

  const veriumLight = useQuery({
    queryKey: coinQueryKey("verium", "light-wallet-exists"),
    queryFn: () => lightWalletExists("verium"),
    enabled: options.includes("verium"),
  });
  const vericoinLight = useQuery({
    queryKey: coinQueryKey("vericoin", "light-wallet-exists"),
    queryFn: () => lightWalletExists("vericoin"),
    enabled: options.includes("vericoin"),
  });

  const veriumWalletFile = useQuery({
    queryKey: coinQueryKey("verium", "wallet-file-status"),
    queryFn: () => tauriWalletFileStatus("verium"),
    enabled: options.includes("verium") && walletMode === "full_node",
  });
  const vericoinWalletFile = useQuery({
    queryKey: coinQueryKey("vericoin", "wallet-file-status"),
    queryFn: () => tauriWalletFileStatus("vericoin"),
    enabled: options.includes("vericoin") && walletMode === "full_node",
  });

  const readyOptions = (coin: CoinId) => ({
    walletMode,
    hasLightWallet: coin === "verium" ? veriumLight.data : vericoinLight.data,
    hasFullNodeWallet: fullNodeWalletExists(
      coin === "verium" ? veriumWalletFile.data : vericoinWalletFile.data,
    ),
  });

  const anyComplete = options.some((coin) =>
    isCoinWalletReady(coin, prefs, readyOptions(coin)),
  );
  const allComplete = options.every((coin) =>
    isCoinWalletReady(coin, prefs, readyOptions(coin)),
  );

  return (
    <div className="flex flex-col gap-5 text-sm">
      <p className="text-fg-muted">
        Choose Verium or Vericoin to set up or continue onboarding. You can
        return here anytime from setup to switch chains or open the dashboard.
      </p>

      <div className="flex flex-col gap-2 rounded-md border border-border bg-bg-subtle p-3">
        <p className="text-xs font-medium text-fg">Appearance</p>
        <ThemeSegmented value={themeMode} onChange={setThemeMode} />
        <p className="text-[11px] text-fg-subtle">
          Auto follows your system light or dark setting.
        </p>
      </div>

      {LIGHT_WALLET_ENABLED && (
        <div className="flex flex-col gap-2 rounded-md border border-border bg-bg-subtle p-3">
          <p className="text-xs font-medium text-fg">Wallet mode (app-wide)</p>
          <div className="grid gap-2 sm:grid-cols-1">
            <button
              type="button"
              className={cn(
                "rounded border p-3 text-left text-xs",
                walletMode === "full_node"
                  ? "border-accent bg-accent/10"
                  : "border-border",
              )}
              onClick={() => onWalletModeChange("full_node")}
            >
              <strong>Full node (recommended)</strong>
              <br />
              {lightWalletCopy.setupFullNodeRecommended}
            </button>
          </div>
          {!showAdvancedLightMode ? (
            <Button
              size="sm"
              variant="ghost"
              className="self-start text-xs"
              onClick={onShowAdvancedLightMode}
            >
              Advanced: light wallet (convenience)
            </Button>
          ) : (
            <div className="flex flex-col gap-2">
              <p className="text-xs text-warning">{lightWalletCopy.lightConvenienceWarning}</p>
              <button
                type="button"
                className={cn(
                  "rounded border p-3 text-left text-xs",
                  walletMode === "light"
                    ? "border-warning bg-warning/10"
                    : "border-border",
                )}
                onClick={() => onWalletModeChange("light")}
              >
                <strong>Light wallet (convenience)</strong>
                <br />
                {lightWalletCopy.setupWelcomeLight}
              </button>
            </div>
          )}
        </div>
      )}

      <div className="grid gap-3 sm:grid-cols-2">
        {options.map((coin) => (
          <CoinHubCard
            key={coin}
            coin={coin}
            prefs={prefs}
            walletMode={walletMode}
            onSelect={onSelectCoin}
            opening={openingCoin === coin}
          />
        ))}
      </div>

      {anyComplete && (
        <div className="flex flex-wrap items-center gap-2 border-t border-border pt-4">
          <Button size="sm" variant="secondary" onClick={onOpenDashboard}>
            {allComplete
              ? "Open dashboard"
              : "Open dashboard (skip remaining setup)"}
          </Button>
          {!allComplete && (
            <p className="text-xs text-fg-subtle">
              You can finish the other chain later from Settings → Chains or by
              returning here.
            </p>
          )}
        </div>
      )}
    </div>
  );
}
