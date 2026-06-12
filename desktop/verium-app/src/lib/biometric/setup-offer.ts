import { create } from 'zustand';
import type { CoinId } from '@/lib/coin/profile';
import type { UserPreferences } from '@/lib/user-preferences';
import type { BiometricStatus } from '@/lib/biometric/client';

function isBiometricUnlockReady(status: BiometricStatus | undefined) {
  return Boolean(status?.available && status.enabled && status.configured);
}

export interface BiometricSetupPending {
  coin: CoinId;
  passphrase: string;
}

interface BiometricSetupOfferState {
  pending: BiometricSetupPending | null;
  request: (coin: CoinId, passphrase: string) => void;
  clear: () => void;
}

export const useBiometricSetupOffer = create<BiometricSetupOfferState>((set) => ({
  pending: null,
  request: (coin, passphrase) => set({ pending: { coin, passphrase } }),
  clear: () => set({ pending: null }),
}));

export function requestBiometricSetupOffer(coin: CoinId, passphrase: string) {
  useBiometricSetupOffer.getState().request(coin, passphrase);
}

/** Whether to show the one-time Face ID / Touch ID setup prompt after unlock or wallet creation. */
export function shouldOfferBiometricSetup(
  status: BiometricStatus | undefined,
  prefs: UserPreferences,
  prefsLoaded: boolean
): boolean {
  if (!prefsLoaded) return false;
  if (!status?.available) return false;
  if (isBiometricUnlockReady(status)) return false;
  if (prefs.biometric_unlock_prompt_dismissed) return false;
  return true;
}
