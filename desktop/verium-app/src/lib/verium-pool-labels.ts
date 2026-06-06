import {
  VERIUM_POOL_DISPLAY_NAME,
  VERIUM_POOL_PAYOUT_ADDRESS,
} from "@/lib/verium-pool";

export function isVeriumPoolPayoutAddress(
  address: string | null | undefined,
): boolean {
  return (
    typeof address === "string" &&
    address.trim() === VERIUM_POOL_PAYOUT_ADDRESS
  );
}

export function isVeriumPoolMinerAddress(
  minerAddress: string | null | undefined,
): boolean {
  return isVeriumPoolPayoutAddress(minerAddress);
}

export { VERIUM_POOL_DISPLAY_NAME };
