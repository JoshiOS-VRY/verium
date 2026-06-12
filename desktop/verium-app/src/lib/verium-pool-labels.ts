import { VERIUM_POOL_DISPLAY_NAME, VERIUM_POOL_PAYOUT_ADDRESS } from '@/lib/verium-pool';

export function isVeriumPoolPayoutAddress(address: string | null | undefined): boolean {
  return typeof address === 'string' && address.trim() === VERIUM_POOL_PAYOUT_ADDRESS;
}

function isVeriumPoolDisplayLabel(label: string): boolean {
  const trimmed = label.trim();
  if (trimmed === VERIUM_POOL_DISPLAY_NAME) return true;
  const normalized = trimmed.toLowerCase().replace(/\s+/g, '');
  return normalized === 'veriumpool' || normalized === 'vrmpool';
}

export function isVeriumPoolMinerAddress(minerAddress: string | null | undefined): boolean {
  if (!minerAddress?.trim()) return false;
  const trimmed = minerAddress.trim();
  return isVeriumPoolPayoutAddress(trimmed) || isVeriumPoolDisplayLabel(trimmed);
}

/** Explorer links must use the payout address, not the display label. */
export function resolveVeriumMinerExplorerAddress(
  minerAddress: string | null | undefined
): string | null {
  if (!minerAddress?.trim()) return null;
  const trimmed = minerAddress.trim();
  if (isVeriumPoolPayoutAddress(trimmed)) return trimmed;
  if (isVeriumPoolDisplayLabel(trimmed)) return VERIUM_POOL_PAYOUT_ADDRESS;
  return trimmed;
}

export { VERIUM_POOL_DISPLAY_NAME };
