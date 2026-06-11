import type { CoinId } from '@/lib/coin/profile';
import type { WalletFileStatus } from '@/lib/rpc/client';
import type { UserPreferences } from '@/lib/user-preferences';
import type { WalletProfile } from '@/lib/wallet-profile';

export type WalletModeChoice = 'light' | 'full_node';

export interface CoinWalletReadyOptions {
  walletMode?: WalletModeChoice;
  hasLightWallet?: boolean;
  hasFullNodeWallet?: boolean;
  lightKeystoreHealth?: WalletProfile['light_keystore_health'];
}

export interface CoinWalletPresence {
  hasLightWallet?: boolean;
  hasFullNodeWallet?: boolean;
}

/** App-wide or hub-local mode: persisted light mode wins; else hub selection. */
export function resolveEffectiveWalletMode(
  persistedMode: WalletModeChoice | undefined,
  localChoice: WalletModeChoice
): WalletModeChoice {
  if (persistedMode === 'light') return 'light';
  if (localChoice === 'light') return 'light';
  return 'full_node';
}

/**
 * Mode to open a coin in. By default only opens when keys exist for
 * `preferredMode`. Pass `allowCrossModeFallback` for coin-switcher paths where
 * an inherited light default should not block an existing full-node wallet.
 */
export function resolveCoinOpenMode(
  preferredMode: WalletModeChoice,
  presence: CoinWalletPresence,
  options?: { allowCrossModeFallback?: boolean }
): WalletModeChoice | null {
  const { hasLightWallet, hasFullNodeWallet } = presence;
  if (preferredMode === 'full_node' && hasFullNodeWallet) return 'full_node';
  if (preferredMode === 'light' && hasLightWallet) return 'light';
  if (!options?.allowCrossModeFallback) return null;
  if (hasFullNodeWallet && !hasLightWallet) return 'full_node';
  if (hasLightWallet && !hasFullNodeWallet) return 'light';
  if (hasLightWallet && hasFullNodeWallet) return preferredMode;
  return null;
}

export function profileWalletPresenceFromProfile(profile: WalletProfile): CoinWalletPresence {
  return {
    hasLightWallet: profile.keys_present.light,
    hasFullNodeWallet: profile.keys_present.full_node || profile.legacy.detected,
  };
}

/** True when any persisted wallet keys can be opened (decryptable for light). */
export function isProfileOpenable(profile: WalletProfile | undefined | null): boolean {
  if (!profile) return false;
  if (profile.openable) return true;
  if (
    profile.light_keystore_health === 'unreadable' &&
    profile.keys_present.light &&
    !profile.keys_present.full_node &&
    !profile.legacy.detected
  ) {
    return false;
  }
  return (
    resolveCoinOpenMode(
      profile.mode as WalletModeChoice,
      profileWalletPresenceFromProfile(profile),
      { allowCrossModeFallback: true }
    ) !== null
  );
}

export function isLightKeystoreUnreadable(
  profile: Pick<WalletProfile, 'light_keystore_health' | 'keys_present'> | null | undefined
): boolean {
  return profile?.light_keystore_health === 'unreadable' && profile.keys_present.light === true;
}

/** Light recovery is required only when light keys are broken *and* full-node is not the viable path. */
export function needsLightWalletRecovery(
  profile:
    | Pick<WalletProfile, 'light_keystore_health' | 'keys_present' | 'mode' | 'ready' | 'legacy'>
    | null
    | undefined,
  walletMode?: WalletModeChoice
): boolean {
  if (!isLightKeystoreUnreadable(profile)) return false;
  const presence = profile
    ? profileWalletPresenceFromProfile(profile as WalletProfile)
    : { hasLightWallet: false, hasFullNodeWallet: false };
  const effectiveMode =
    walletMode ?? (profile?.mode as WalletModeChoice | undefined) ?? 'full_node';
  if (presence.hasFullNodeWallet && effectiveMode === 'full_node') {
    return false;
  }
  if (profile?.mode === 'full_node' && profile.ready) {
    return false;
  }
  return true;
}

export interface HubCardState {
  statusLabel: string;
  actionLabel: 'Open' | 'Set up' | 'Import phrase';
  ready: boolean;
}

/** Hub card labels — cross-mode storage and openability independent of hub mode. */
export function getHubCardState(options: {
  checking: boolean;
  walletMode: WalletModeChoice;
  presence: CoinWalletPresence;
  profile?: WalletProfile | null;
}): HubCardState {
  const { checking, walletMode, presence, profile } = options;
  if (checking) {
    return { statusLabel: 'Checking…', actionLabel: 'Set up', ready: false };
  }

  const readyFromHub = isCoinWalletReady(
    'verium',
    { setup_completed: false },
    {
      walletMode,
      hasLightWallet: presence.hasLightWallet,
      hasFullNodeWallet: presence.hasFullNodeWallet,
      lightKeystoreHealth: profile?.light_keystore_health,
    }
  );
  const ready =
    readyFromHub || (profile?.ready === true && (profile.mode as WalletModeChoice) === walletMode);
  const hasStored = coinHasStoredWallet(presence);
  const unreadable = needsLightWalletRecovery(profile ?? undefined, walletMode);
  const storedElsewhere =
    !ready &&
    !unreadable &&
    hasStored &&
    (walletMode === 'light'
      ? presence.hasFullNodeWallet === true && presence.hasLightWallet !== true
      : presence.hasLightWallet === true && presence.hasFullNodeWallet !== true);
  const openable = profile ? isProfileOpenable(profile) : false;

  let statusLabel: string;
  if (unreadable) {
    statusLabel = 'Recovery required';
  } else if (ready) {
    statusLabel = 'Ready';
  } else if (storedElsewhere) {
    statusLabel = walletMode === 'light' ? 'Full node on device' : 'Light wallet on device';
  } else {
    statusLabel = 'Not set up';
  }

  const actionLabel: 'Open' | 'Set up' | 'Import phrase' = unreadable
    ? 'Import phrase'
    : ready || openable
      ? 'Open'
      : 'Set up';

  return { statusLabel, actionLabel, ready };
}

export function coinHasStoredWallet(presence: CoinWalletPresence): boolean {
  return presence.hasLightWallet === true || presence.hasFullNodeWallet === true;
}

/** True if wallet.dat (or a legacy Qt wallet) exists for this chain. */
export function fullNodeWalletExists(
  status?: Pick<WalletFileStatus, 'exists' | 'legacy_wallet_detected'> | null
): boolean {
  return status?.exists === true || status?.legacy_wallet_detected === true;
}

/** True if any enabled chain still needs first-run setup. */
export function anyEnabledCoinSetupIncomplete(
  enabledCoins: CoinId[],
  prefs: Pick<UserPreferences, 'setup_completed' | 'setup_completed_by_coin'>
): boolean {
  return enabledCoins.some((coin) => !isCoinSetupComplete(coin, prefs));
}

/**
 * True when the selected wallet mode has persisted keys on this device.
 * Does not use setup_completed — that flag can be set from full-node onboarding
 * while the app is later switched to light mode without a light keystore.
 */
export function isCoinWalletReady(
  coin: CoinId,
  prefs: Pick<UserPreferences, 'setup_completed' | 'setup_completed_by_coin'>,
  options?: CoinWalletReadyOptions | boolean
): boolean {
  void coin;
  void prefs;

  const opts: CoinWalletReadyOptions =
    typeof options === 'boolean' ? { hasLightWallet: options } : (options ?? {});

  const mode = opts.walletMode ?? 'full_node';
  if (mode === 'light') {
    if (opts.lightKeystoreHealth === 'unreadable') return false;
    return opts.hasLightWallet === true;
  }
  return opts.hasFullNodeWallet === true;
}

/** True when the user finished (or migrated) first-run setup for this chain. */
export function isCoinSetupComplete(
  coin: CoinId,
  prefs: Pick<UserPreferences, 'setup_completed' | 'setup_completed_by_coin'>,
  presence?: CoinWalletPresence
): boolean {
  if (coinHasStoredWallet(presence ?? {})) return true;
  const perCoin = prefs.setup_completed_by_coin?.[coin];
  if (perCoin === true) return true;
  if (perCoin === false) return false;
  // Legacy single flag: only treat Verium as done (pre–dual-chain installs).
  if (prefs.setup_completed && coin === 'verium') return true;
  return false;
}

/** Merge partial prefs to mark one chain's setup wizard finished. */
export function coinSetupCompletePatch(
  coin: CoinId,
  prefs: UserPreferences
): Partial<UserPreferences> {
  return {
    setup_completed_by_coin: {
      ...prefs.setup_completed_by_coin,
      [coin]: true,
    },
    setup_completed: prefs.setup_completed || coin === 'verium',
  };
}
