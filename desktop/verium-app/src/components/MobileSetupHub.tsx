import { ArrowRight, Loader2, Plus, RefreshCw, Wallet } from 'lucide-react';
import { ALL_COINS, COIN_LOGO_URLS, COIN_PROFILES, type CoinId } from '@/lib/coin/profile';
import { useEnabledCoins } from '@/lib/coin/context';
import { lightWalletCopy } from '@/lib/light-wallet/copy';
import { isProfileOpenable, needsLightWalletRecovery } from '@/lib/setup';
import { useSetupHubProfile } from '@/hooks/useSetupHubProfiles';
import { Button } from '@/components/ui/Button';
import { cn } from '@/lib/utils';

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
    <article
      className={cn(
        'mobile-setup-coin-card flex min-w-0 flex-col gap-4 overflow-hidden rounded-2xl border p-4 shadow-sm transition-shadow',
        ready
          ? 'border-success/30 bg-gradient-to-br from-success/5 via-bg-panel to-bg-panel'
          : 'border-border bg-gradient-to-br from-bg-panel via-bg-panel to-bg-subtle/40'
      )}
    >
      <div className="flex min-w-0 items-start gap-3">
        <div className="relative shrink-0">
          <img
            src={COIN_LOGO_URLS[coin]}
            alt=""
            className="h-14 w-14 rounded-2xl object-contain ring-1 ring-border/60"
          />
          {ready && (
            <span className="absolute -bottom-1 -right-1 h-3.5 w-3.5 rounded-full border-2 border-bg-panel bg-success" />
          )}
        </div>
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <h3 className="text-lg font-semibold tracking-tight text-fg">
              {coinProfile.displayName}
            </h3>
            <span
              className={cn(
                'rounded-full px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide',
                coinProfile.accentClass
              )}
            >
              {coinProfile.symbol}
            </span>
          </div>
          <p className="mt-1 text-xs leading-relaxed text-fg-subtle">{coinProfile.tagline}</p>
          <span
            className={cn(
              'mt-2 inline-flex rounded-full px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide',
              ready
                ? 'bg-success/10 text-success'
                : checking
                  ? 'bg-bg-subtle text-fg-muted'
                  : 'bg-bg-subtle text-fg-muted'
            )}
          >
            {checking
              ? 'Checking…'
              : ready
                ? lightWalletCopy.mobileOnboardingReady
                : lightWalletCopy.mobileOnboardingNotSetUp}
          </span>
        </div>
      </div>

      {needsRestore && (
        <p className="rounded-xl border border-accent/25 bg-accent/5 px-3 py-2.5 text-xs leading-relaxed text-fg-muted">
          {lightWalletCopy.mobileOnboardingRestoreHint}
        </p>
      )}

      {ready ? (
        <Button
          type="button"
          className="h-12 w-full rounded-xl text-base font-semibold"
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
            className="h-11 w-full rounded-xl"
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
            className="h-11 w-full rounded-xl"
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

  const veriumProfile = useSetupHubProfile('verium', options.includes('verium'));
  const vericoinProfile = useSetupHubProfile('vericoin', options.includes('vericoin'));

  const profileByCoin = {
    verium: veriumProfile,
    vericoin: vericoinProfile,
  } as const;

  const anyReady = options.some((coin) => {
    const profile = profileByCoin[coin].data;
    return profile && isProfileOpenable(profile) && profile.ready;
  });

  return (
    <div className="mobile-setup-hub flex min-w-0 flex-col gap-5">
      <header className="mobile-setup-hero rounded-2xl border border-border/80 bg-gradient-to-br from-accent/10 via-bg-panel to-bg-subtle/60 px-4 py-5 text-center shadow-sm">
        <span className="mx-auto flex h-12 w-12 items-center justify-center rounded-2xl bg-accent/15 text-accent">
          <Wallet className="h-6 w-6" aria-hidden />
        </span>
        <h1 className="mt-3 text-xl font-bold tracking-tight text-fg">Vericonomy Wallet</h1>
        <p className="mt-2 text-sm leading-relaxed text-fg-muted">
          {lightWalletCopy.mobileOnboardingWelcome}
        </p>
      </header>

      <p className="rounded-xl border border-accent/20 bg-accent/5 px-3 py-3 text-xs leading-relaxed text-fg-muted">
        {lightWalletCopy.mobileFundsDisclaimer}
      </p>

      <div className="grid min-w-0 gap-4">
        {options.map((coin) => {
          const query = profileByCoin[coin];
          const profile = query.data;
          const ready =
            !!profile &&
            profile.ready &&
            isProfileOpenable(profile) &&
            !needsLightWalletRecovery(profile, 'light');
          const needsRestore = needsLightWalletRecovery(profile, 'light');

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
