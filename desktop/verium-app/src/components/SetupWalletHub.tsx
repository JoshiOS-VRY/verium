import { AlertTriangle, ArrowRight, CheckCircle2, Circle, Loader2 } from "lucide-react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import {
  ALL_COINS,
  COIN_LOGO_URLS,
  COIN_PROFILES,
  coinQueryKey,
  type CoinId,
} from "@/lib/coin/profile";
import { useEnabledCoins } from "@/lib/coin/context";
import { lightWalletCopy } from "@/lib/light-wallet/copy";
import {
  getHubCardState,
  needsLightWalletRecovery,
  isProfileOpenable,
  type WalletModeChoice,
} from "@/lib/setup";
import { LIGHT_WALLET_ENABLED } from "@/lib/features";
import {
  profileWalletPresence,
  useSetupHubProfile,
} from "@/hooks/useSetupHubProfiles";
import { ThemeSegmented } from "@/components/ThemeSegmented";
import { Button } from "@/components/ui/Button";
import { useTheme } from "@/hooks/useTheme";
import { cn } from "@/lib/utils";
import {
  secretStoreQuarantineOrphaned,
  secretStoreStatus,
  type WalletProfile,
} from "@/lib/wallet-profile";

interface SetupWalletHubProps {
  walletMode: WalletModeChoice;
  /** iOS/Android — Electrum light wallet only. */
  mobileOnly?: boolean;
  onWalletModeChange: (mode: WalletModeChoice) => void;
  /** @deprecated light mode is now shown inline; retained for caller compat. */
  showAdvancedLightMode?: boolean;
  /** @deprecated light mode is now shown inline; retained for caller compat. */
  onShowAdvancedLightMode?: () => void;
  onSelectCoin: (coin: CoinId) => void | Promise<void>;
  onOpenDashboard: () => void;
  openingCoin?: CoinId | null;
}

function CoinHubCard({
  coin,
  walletMode,
  checking,
  profile,
  errorMessage,
  onSelect,
  opening,
}: {
  coin: CoinId;
  walletMode: WalletModeChoice;
  checking: boolean;
  profile?: WalletProfile;
  errorMessage?: string | null;
  onSelect: (coin: CoinId) => void | Promise<void>;
  opening: boolean;
}) {
  const coinProfile = COIN_PROFILES[coin];
  const presence = profileWalletPresence(profile);
  const { statusLabel, actionLabel, ready } = getHubCardState({
    checking,
    walletMode,
    presence,
    profile,
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
            <span className="font-semibold text-fg">{coinProfile.displayName}</span>
            <span
              className={cn(
                "rounded border px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide",
                coinProfile.accentClass,
              )}
            >
              {coinProfile.symbol}
            </span>
          </div>
          <p className="mt-0.5 text-xs text-fg-subtle">{coinProfile.tagline}</p>
        </div>
      </div>
      {errorMessage && (
        <p className="flex items-start gap-1.5 text-[11px] text-danger">
          <AlertTriangle className="mt-0.5 h-3 w-3 shrink-0" />
          {errorMessage}
        </p>
      )}
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
          {statusLabel}
        </span>
        <span className="flex items-center gap-1 font-medium text-accent">
          {opening ? (
            <>
              Opening…
              <Loader2 className="h-3.5 w-3.5 animate-spin" />
            </>
          ) : (
            <>
              {actionLabel}
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
  mobileOnly = false,
  onWalletModeChange,
  onSelectCoin,
  onOpenDashboard,
  openingCoin = null,
}: SetupWalletHubProps) {
  const queryClient = useQueryClient();
  const [quarantining, setQuarantining] = useState(false);
  const secretStore = useQuery({
    queryKey: ["secret-store-status"],
    queryFn: secretStoreStatus,
    staleTime: 30_000,
  });
  const enabledCoins = useEnabledCoins();
  const { mode: themeMode, setMode: setThemeMode } = useTheme();
  const options = ALL_COINS.filter((coin) => enabledCoins.includes(coin));

  const veriumProfile = useSetupHubProfile(
    "verium",
    options.includes("verium"),
  );
  const vericoinProfile = useSetupHubProfile(
    "vericoin",
    options.includes("vericoin"),
  );

  const profileByCoin: Record<
    CoinId,
    ReturnType<typeof useSetupHubProfile>
  > = {
    verium: veriumProfile,
    vericoin: vericoinProfile,
  };

  const profileLoadError = options.some((coin) => profileByCoin[coin].isError);

  const anyComplete = options.some(
    (coin) =>
      profileByCoin[coin].data &&
      isProfileOpenable(profileByCoin[coin].data) &&
      !needsLightWalletRecovery(profileByCoin[coin].data, walletMode),
  );
  const allComplete = options.every(
    (coin) =>
      profileByCoin[coin].data &&
      isProfileOpenable(profileByCoin[coin].data) &&
      !needsLightWalletRecovery(profileByCoin[coin].data, walletMode),
  );

  const profileErrorMessage =
    "Could not read wallet status. Restart the app or recover from your recovery phrase if encrypted settings cannot be unlocked.";

  const anyNeedsLightRecovery = options.some((coin) =>
    needsLightWalletRecovery(profileByCoin[coin].data, walletMode),
  );

  async function handleQuarantineOrphaned() {
    const confirmed = window.confirm(
      "This moves encrypted wallet files aside and creates a new Windows Credential Manager key. " +
        "Existing light wallets will not unlock until you import your recovery phrase. Continue?",
    );
    if (!confirmed) return;
    setQuarantining(true);
    try {
      await secretStoreQuarantineOrphaned();
      await secretStore.refetch();
      for (const coin of options) {
        await queryClient.invalidateQueries({
          queryKey: coinQueryKey(coin, "wallet-profile"),
        });
      }
    } catch (e) {
      console.warn("secret store quarantine failed", e);
      window.alert(
        e instanceof Error ? e.message : "Could not reset encrypted storage.",
      );
    } finally {
      setQuarantining(false);
    }
  }

  return (
    <div className="flex flex-col gap-5 text-sm">
      <p className="text-fg-muted">
        Choose Verium or Vericoin to set up or continue onboarding. You can
        return here anytime from setup to switch chains or open the dashboard.
      </p>

      {profileLoadError && (
        <div className="flex items-start gap-2 rounded-md border border-danger/40 bg-danger/10 p-3 text-xs text-fg-muted">
          <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0 text-danger" />
          <p>{profileErrorMessage}</p>
        </div>
      )}

      {(secretStore.data?.orphaned || anyNeedsLightRecovery) && (
        <div className="flex flex-col gap-2 rounded-md border border-warning/40 bg-warning/10 p-3 text-xs text-fg-muted">
          <div className="flex items-start gap-2">
            <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0 text-warning" />
            <div className="flex flex-col gap-1.5">
              <p className="font-medium text-fg">
                Encrypted wallet data cannot be unlocked
              </p>
              <p>
                Windows Credential Manager cannot decrypt older app settings.
                Your light wallet is stored separately and is not affected once
                you import your recovery phrase below.
              </p>
            </div>
          </div>
          {secretStore.data?.orphaned && (
            <Button
              size="sm"
              variant="secondary"
              className="self-start"
              disabled={quarantining}
              onClick={() => void handleQuarantineOrphaned()}
            >
              {quarantining ? "Resetting…" : "Reset encrypted storage (advanced)"}
            </Button>
          )}
        </div>
      )}

      <div className="flex flex-col gap-2 rounded-md border border-border bg-bg-subtle p-3">
        <p className="text-xs font-medium text-fg">Appearance</p>
        <ThemeSegmented value={themeMode} onChange={setThemeMode} />
        <p className="text-[11px] text-fg-subtle">
          Auto follows your system light or dark setting.
        </p>
      </div>

      {mobileOnly && (
        <div className="rounded-md border border-accent/30 bg-accent/5 p-3 text-xs text-fg-muted">
          <p className="font-medium text-fg">Light wallet only</p>
          <p className="mt-1">{lightWalletCopy.setupMobileOnly}</p>
        </div>
      )}

      {LIGHT_WALLET_ENABLED && !mobileOnly && (
        <div className="flex flex-col gap-2 rounded-md border border-border bg-bg-subtle p-3">
          <p className="text-xs font-medium text-fg">Default wallet mode</p>
          <p className="text-[11px] text-fg-subtle">
            Sets the tier for new chains. You can switch any chain
            independently later from its wallet settings.
          </p>
          <div className="grid gap-2 sm:grid-cols-2">
            <button
              type="button"
              className={cn(
                "rounded border p-3 text-left text-xs",
                walletMode === "full_node"
                  ? "border-accent bg-accent/10"
                  : "border-border hover:border-accent/50",
              )}
              onClick={() => onWalletModeChange("full_node")}
            >
              <strong>Full node (recommended)</strong>
              <br />
              {lightWalletCopy.setupFullNodeRecommended}
            </button>
            <button
              type="button"
              className={cn(
                "rounded border p-3 text-left text-xs",
                walletMode === "light"
                  ? "border-warning bg-warning/10"
                  : "border-border hover:border-accent/50",
              )}
              onClick={() => onWalletModeChange("light")}
            >
              <strong>Light wallet (convenience)</strong>
              <br />
              {lightWalletCopy.setupWelcomeLight}
            </button>
          </div>
          {walletMode === "light" && (
            <p className="text-xs text-warning">
              {lightWalletCopy.lightConvenienceWarning}
            </p>
          )}
        </div>
      )}

      <div className="grid gap-3 sm:grid-cols-2">
        {options.map((coin) => {
          const query = profileByCoin[coin];
          return (
            <CoinHubCard
              key={coin}
              coin={coin}
              walletMode={walletMode}
              checking={query.isLoading}
              profile={query.data}
              errorMessage={
                query.isError
                  ? profileErrorMessage
                  : query.data &&
                      needsLightWalletRecovery(query.data, walletMode)
                    ? "Encrypted keys unreadable — import recovery phrase."
                    : null
              }
              onSelect={onSelectCoin}
              opening={openingCoin === coin}
            />
          );
        })}
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
