import type { CoinId } from "@/lib/coin/profile";
import type { WalletFileStatus } from "@/lib/rpc/client";
import type { UserPreferences } from "@/lib/user-preferences";

export type WalletModeChoice = "light" | "full_node";

export interface CoinWalletReadyOptions {
  walletMode?: WalletModeChoice;
  hasLightWallet?: boolean;
  hasFullNodeWallet?: boolean;
}

/** True if wallet.dat (or a legacy Qt wallet) exists for this chain. */
export function fullNodeWalletExists(
  status?: Pick<WalletFileStatus, "exists" | "legacy_wallet_detected"> | null,
): boolean {
  return status?.exists === true || status?.legacy_wallet_detected === true;
}

/** True if any enabled chain still needs first-run setup. */
export function anyEnabledCoinSetupIncomplete(
  enabledCoins: CoinId[],
  prefs: Pick<UserPreferences, "setup_completed" | "setup_completed_by_coin">,
): boolean {
  return enabledCoins.some((coin) => !isCoinSetupComplete(coin, prefs));
}

/**
 * Chain is ready to open from the hub: wizard marked complete, or the selected
 * wallet mode already has persisted keys (light keystore vs local wallet.dat).
 */
export function isCoinWalletReady(
  coin: CoinId,
  prefs: Pick<UserPreferences, "setup_completed" | "setup_completed_by_coin">,
  options?: CoinWalletReadyOptions | boolean,
): boolean {
  if (isCoinSetupComplete(coin, prefs)) return true;

  const opts: CoinWalletReadyOptions =
    typeof options === "boolean" ? { hasLightWallet: options } : (options ?? {});

  const mode = opts.walletMode ?? "full_node";
  if (mode === "light") {
    return opts.hasLightWallet === true;
  }
  return opts.hasFullNodeWallet === true;
}

/** True when the user finished (or migrated) first-run setup for this chain. */
export function isCoinSetupComplete(
  coin: CoinId,
  prefs: Pick<UserPreferences, "setup_completed" | "setup_completed_by_coin">,
): boolean {
  const perCoin = prefs.setup_completed_by_coin?.[coin];
  if (perCoin === true) return true;
  if (perCoin === false) return false;
  // Legacy single flag: only treat Verium as done (pre–dual-chain installs).
  if (prefs.setup_completed && coin === "verium") return true;
  return false;
}

/** Merge partial prefs to mark one chain's setup wizard finished. */
export function coinSetupCompletePatch(
  coin: CoinId,
  prefs: UserPreferences,
): Partial<UserPreferences> {
  return {
    setup_completed_by_coin: {
      ...prefs.setup_completed_by_coin,
      [coin]: true,
    },
    setup_completed: prefs.setup_completed || coin === "verium",
  };
}
