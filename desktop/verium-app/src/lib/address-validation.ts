import type { CoinId } from './coin/profile';
import { sdkProfile } from './coin/sdk-profile';

const BASE58_PATTERN = /^[123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz]+$/;

export function normalizeSendAddress(raw: string): string {
  return raw.trim();
}

function p2pkhRules(coin: CoinId) {
  const profile = sdkProfile(coin);
  return {
    prefix: profile.p2pkh_address_prefix,
    length: profile.p2pkh_address_length,
  };
}

/** Returns a user-facing error message, or `null` when the address format is valid. */
export function validateSendAddress(address: string, coin: CoinId = 'verium'): string | null {
  const trimmed = normalizeSendAddress(address);
  const { prefix, length } = p2pkhRules(coin);

  if (!trimmed) {
    return 'Address is required.';
  }
  if (!trimmed.startsWith(prefix)) {
    return `Address must start with ${prefix}.`;
  }
  if (trimmed.length !== length) {
    return `Address must be exactly ${length} characters.`;
  }
  if (!BASE58_PATTERN.test(trimmed)) {
    return 'Address must contain only letters and numbers (no spaces or symbols).';
  }
  return null;
}
