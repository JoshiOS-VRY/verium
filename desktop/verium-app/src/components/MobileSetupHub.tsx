import { ArrowRight, Loader2, Plus, RefreshCw } from "lucide-react";
import {
  ALL_COINS,
  COIN_LOGO_URLS,
  COIN_PROFILES,
  type CoinId,
} from "@/lib/coin/profile";
import { useEnabledCoins } from "@/lib/coin/context";
import { lightWalletCopy } from "@/lib/light-wallet/copy";
import {
  isProfileOpenable,
  needsLightWalletRecovery,
} from "@/lib/setup";
import { useSetupHubProfile } from "@/hooks/useSetupHubProfiles";
import { Button } from "@/components/ui/Button";
import { cn } from "@/lib/utils";

interface MobileSetupHubProps {
  onCreateWallet: (coin: CoinId) => void | Promise<void>;
  onImportPhrase: (coin: CoinId) => void | Promise<void>;
  onOpenWallet: (coin: CoinId) => void | Promise<void>;
  openingCoin?: CoinId | null;
}

function MobileCoinCard({
  coin,
  checking,
  ready,
  needsRestore,
  onCreate,
  onImport,
  onOpen,
  opening,
}: {
  coin: CoinId;
  checking: boolean;
  ready: boolean;
  needsRestore: boolean;
  onCreate: () => void;
  onImport: () => void;
  onOpen: () => void;
  opening: boolean;
}) {
  const coinProfile = COIN_PROFILES[coin];

  return (
    <article className="flex min-w-0 flex-col gap-4 rounded-2xl border border-border bg-bg-panel/80 p-4 shadow-sm">
      <div className="flex min-w-0 items-start gap-3">
        <img
          src={COIN_LOGO_URLS[coin]}
          alt=""
          className="h-12 w-12 shrink-0 rounded-xl object-contain"
        />
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <h3 className="text-base font-semibold text-fg">
              {coinProfile.displayName}
            </h3>
            <span
              className={cn(
                "rounded border px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide",
                coinProfile.accentClass,
              )}
            >
              {coinProfile.symbol}
            </span>
          </div>
          <p className="mt-1 text-xs text-fg-subtle">{coinProfile.tagline}</p>
          <p
            className={cn(
              "mt-2 text-xs font-medium",
              ready ? "text-success" : "text-fg-muted",
            )}
          >
            {checking
              ? "Checking…"
              : ready
                ? lightWalletCopy.mobileOnboardingReady
                : lightWalletCopy.mobileOnboardingNotSetUp}
          </p>
        </div>
      </div>

      {needsRestore && (
        <p className="rounded-lg border border-accent/25 bg-accent/5 px-3 py-2.5 text-xs text-fg-muted">
          {lightWalletCopy.mobileOnboardingRestoreHint}
        </p>
      )}

      {ready ? (
        <Button
          type="button"
          className="w-full"
          disabled={opening}
          onClick={onOpen}
        >
          {opening ? (
            <>
              Opening…
              <Loader2 className="h-4 w-4 animate-spin" />
            </>
          ) : (
            <>
              Open wallet
              <ArrowRight className="h-4 w-4" />
            </>
          )}
        </Button>
      ) : (
        <div className="flex min-w-0 flex-col gap-2">
          <Button
            type="button"
            className="w-full"
            disabled={opening || checking}
            onClick={onCreate}
          >
            {opening ? (
              <>
                Starting…
                <Loader2 className="h-4 w-4 animate-spin" />
              </>
            ) : (
              <>
                <Plus className="h-4 w-4" />
                Create new wallet
              </>
            )}
          </Button>
          <Button
            type="button"
            variant="secondary"
            className="w-full"
            disabled={opening || checking}
            onClick={onImport}
          >
            <RefreshCw className="h-4 w-4" />
            Import recovery phrase
          </Button>
        </div>
      )}
    </article>
  );
}

export function MobileSetupHub({
  onCreateWallet,
  onImportPhrase,
  onOpenWallet,
  openingCoin = null,
}: MobileSetupHubProps) {
  const enabledCoins = useEnabledCoins();
  const options = ALL_COINS.filter((coin) => enabledCoins.includes(coin));

  const veriumProfile = useSetupHubProfile(
    "verium",
    options.includes("verium"),
  );
  const vericoinProfile = useSetupHubProfile(
    "vericoin",
    options.includes("vericoin"),
  );

  const profileByCoin = {
    verium: veriumProfile,
    vericoin: vericoinProfile,
  } as const;

  const anyReady = options.some((coin) => {
    const profile = profileByCoin[coin].data;
    return profile && isProfileOpenable(profile) && profile.ready;
  });

  return (
    <div className="flex min-w-0 flex-col gap-5">
      <p className="text-sm leading-relaxed text-fg-muted">
        {lightWalletCopy.mobileOnboardingWelcome}
      </p>

      <div className="grid min-w-0 gap-4">
        {options.map((coin) => {
          const query = profileByCoin[coin];
          const profile = query.data;
          const ready =
            !!profile &&
            profile.ready &&
            isProfileOpenable(profile) &&
            !needsLightWalletRecovery(profile, "light");
          const needsRestore = needsLightWalletRecovery(profile, "light");

          return (
            <MobileCoinCard
              key={coin}
              coin={coin}
              checking={query.isLoading}
              ready={ready}
              needsRestore={needsRestore}
              opening={openingCoin === coin}
              onCreate={() => void onCreateWallet(coin)}
              onImport={() => void onImportPhrase(coin)}
              onOpen={() => void onOpenWallet(coin)}
            />
          );
        })}
      </div>

      {anyReady && (
        <p className="border-t border-border pt-4 text-center text-xs text-fg-subtle">
          You can set up the other chain whenever you are ready.
        </p>
      )}
    </div>
  );
}
