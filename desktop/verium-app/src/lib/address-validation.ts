/** P2PKH Base58Check length for Verium / Vericoin (version byte 70 → leading `V`). */
export const P2PKH_ADDRESS_LENGTH = 34;

const BASE58_PATTERN = /^[123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz]+$/;

export function normalizeSendAddress(raw: string): string {
  return raw.trim();
}

/** Returns a user-facing error message, or `null` when the address format is valid. */
export function validateSendAddress(address: string): string | null {
  const trimmed = normalizeSendAddress(address);
  if (!trimmed) {
    return "Address is required.";
  }
  if (!trimmed.startsWith("V")) {
    return "Address must start with V.";
  }
  if (trimmed.length !== P2PKH_ADDRESS_LENGTH) {
    return `Address must be exactly ${P2PKH_ADDRESS_LENGTH} characters.`;
  }
  if (!BASE58_PATTERN.test(trimmed)) {
    return "Address must contain only letters and numbers (no spaces or symbols).";
  }
  return null;
}
