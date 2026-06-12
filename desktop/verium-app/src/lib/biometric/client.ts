import { invoke } from '@tauri-apps/api/core';
import type { CoinId } from '@/lib/coin/profile';
import { rpcUnlockTimeoutSeconds } from '@/lib/wallet-unlock';

export interface BiometricStatus {
  available: boolean;
  biometry_type: number | null;
  enabled: boolean;
  configured: boolean;
}

/** Face ID / Touch ID availability and whether unlock is enabled for this chain. */
export async function biometricUnlockStatus(coin: CoinId): Promise<BiometricStatus> {
  const backend = await invoke<{ enabled: boolean; configured: boolean }>(
    'biometric_unlock_status',
    { coin }
  );
  let available = false;
  let biometry_type: number | null = null;
  try {
    const { checkStatus } = await import('@tauri-apps/plugin-biometric');
    const status = await checkStatus();
    available = status.isAvailable;
    biometry_type = status.biometryType;
  } catch {
    available = false;
  }
  return {
    ...backend,
    available,
    biometry_type,
  };
}

/** Verify passphrase, store in secure keychain, and enable the preference. */
export async function biometricUnlockEnable(coin: CoinId, passphrase: string): Promise<void> {
  await invoke('biometric_unlock_enable', { coin, passphrase });
}

/** Remove stored passphrase and disable the preference. */
export async function biometricUnlockDisable(coin: CoinId): Promise<void> {
  await invoke('biometric_unlock_disable', { coin });
}

/**
 * Unlock the light wallet using the passphrase saved for biometric unlock.
 * Call only after a successful `authenticate()` from the biometric plugin.
 */
export async function biometricUnlockWallet(
  coin: CoinId,
  seconds = rpcUnlockTimeoutSeconds()
): Promise<void> {
  await invoke('biometric_unlock_wallet', { coin, seconds });
}

export async function promptBiometricUnlock(reason: string): Promise<void> {
  const { authenticate } = await import('@tauri-apps/plugin-biometric');
  await authenticate(reason, {
    allowDeviceCredential: true,
    cancelTitle: 'Cancel',
    fallbackTitle: 'Use passcode',
  });
}

export function biometryLabel(type: number | null): string {
  switch (type) {
    case 2:
      return 'Face ID';
    case 1:
      return 'Touch ID';
    default:
      return 'Biometrics';
  }
}
