import { invoke } from "@tauri-apps/api/core";
import type { CoinId } from "@/lib/coin/profile";
import type { WalletMode } from "@/lib/light-wallet/client";

/**
 * Unified per-coin readiness model. Mirrors the Rust `WalletProfile` returned by
 * the `wallet_profile` command. This is the single source of truth for "does
 * this coin still need setup?" — replacing the previously-split
 * `isCoinWalletReady` / `isCoinSetupComplete` helpers that could disagree.
 */

export type OnboardingIntent =
  | "fresh_install"
  | "legacy_upgrade"
  | "existing_unlock"
  | "light_continue"
  | "cross_mode_migration";

export type OnboardingPhase = "not_started" | "in_progress" | "complete";

export interface OnboardingCheckpoint {
  phase: OnboardingPhase;
  intent?: OnboardingIntent | null;
  /** Step id the wizard should resume at. */
  step?: string | null;
  /** Detected legacy wallet.dat path captured at classification time. */
  legacy_path?: string | null;
}

export type LightKeystoreHealth = "ok" | "unreadable" | "missing";

export interface WalletProfile {
  coin: CoinId;
  mode: WalletMode;
  keys_present: {
    full_node: boolean;
    light: boolean;
  };
  legacy: {
    detected: boolean;
    path?: string | null;
  };
  hd: {
    has_mnemonic_backup: boolean;
  };
  onboarding: OnboardingCheckpoint;
  intent: OnboardingIntent;
  /** True when keys exist for the active mode — the only gate for the dashboard. */
  ready: boolean;
  /** True when any persisted wallet keys exist for this coin (either mode). */
  openable: boolean;
  light_keystore_health: LightKeystoreHealth;
}

export async function tauriWalletProfile(coin: CoinId): Promise<WalletProfile> {
  return invoke<WalletProfile>("wallet_profile", { coin });
}

export async function onboardingCheckpointGet(
  coin: CoinId,
): Promise<OnboardingCheckpoint> {
  return invoke<OnboardingCheckpoint>("onboarding_checkpoint_get", { coin });
}

export async function onboardingCheckpointSet(
  coin: CoinId,
  checkpoint: OnboardingCheckpoint,
): Promise<void> {
  return invoke("onboarding_checkpoint_set", { coin, checkpoint });
}

export async function onboardingMarkComplete(coin: CoinId): Promise<void> {
  return invoke("onboarding_mark_complete", { coin });
}

/** Legacy datadir the user can adopt (folder holding the detected wallet.dat). */
export async function legacyDatadirCandidate(
  coin: CoinId,
): Promise<string | null> {
  return invoke<string | null>("legacy_datadir_candidate", { coin });
}

/** Request a one-shot `-upgradewallet` and restart so `sethdseed` can run. */
export async function legacyRequestHdUpgrade(coin: CoinId): Promise<void> {
  return invoke("legacy_request_hd_upgrade", { coin });
}

export interface CoinStorageDiagnostic {
  coin: string;
  manifest: boolean;
  light_cache: boolean;
  enc_blob: boolean;
  decrypted_keystore: boolean;
  wallet_dat: boolean;
  legacy_wallet: boolean;
  light_on_disk: boolean;
  light_keystore_health: LightKeystoreHealth;
}

export interface WalletStorageDiagnostics {
  app_config_base: string;
  secret_store_orphaned: boolean;
  prefs_enc_readable: boolean;
  keystore_enc_readable: boolean;
  coins: CoinStorageDiagnostic[];
}

export interface SecretStoreStatus {
  orphaned: boolean;
  message?: string | null;
}

export async function secretStoreStatus(): Promise<SecretStoreStatus> {
  return invoke<SecretStoreStatus>("secret_store_status");
}

/** Quarantine unreadable encrypted blobs and create a fresh CM master key. */
export async function secretStoreQuarantineOrphaned(): Promise<string> {
  return invoke<string>("secret_store_quarantine_orphaned");
}

export async function walletStorageDiagnostics(): Promise<WalletStorageDiagnostics> {
  return invoke<WalletStorageDiagnostics>("wallet_storage_diagnostics");
}

/** A coin is ready iff keys exist for its active wallet mode. */
export function isProfileReady(profile: WalletProfile | undefined | null): boolean {
  return profile?.ready === true;
}

/** Whether this coin still needs the setup wizard (not ready and not complete). */
export function profileNeedsSetup(
  profile: WalletProfile | undefined | null,
): boolean {
  if (!profile) return true;
  if (profile.ready) return false;
  return profile.onboarding.phase !== "complete";
}
