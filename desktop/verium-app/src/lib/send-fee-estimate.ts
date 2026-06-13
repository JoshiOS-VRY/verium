/** Typical 1-input P2PKH send (version + nTime + inputs + outputs). */
export function estimatedTxBytes(inputCount = 1, outputCount = 2): number {
  return 10 + inputCount * 148 + outputCount * 34;
}

/** @deprecated use estimatedTxBytes(1, 2) */
export const ESTIMATED_TX_BYTES = estimatedTxBytes(1, 2);

/** Minimum fee rate on Verium VIP1+ networks (0.001 VRM/kB). */
export const MIN_TX_FEE_RATE_VRM_PER_KB = 0.001;

/** Verium `CFeeRate::GetFee` with start-fee enabled (matches Rust `verium_fee_for_size`). */
export function veriumFeeForSizeVrm(ratePerKb: number, bytes: number): number {
  const rateSats = Math.round(ratePerKb * 100_000_000);
  const minRateSats = Math.round(MIN_TX_FEE_RATE_VRM_PER_KB * 100_000_000);
  const effectiveRate = Math.max(rateSats, minRateSats);
  let feeSats = effectiveRate * Math.floor(bytes / 1000) + effectiveRate;
  if (feeSats === 0 && bytes > 0) feeSats = 1;
  return feeSats / 100_000_000;
}

export function estimateSendFee(
  feeRatePerKb: number,
  transactionCount = 1,
  outputCount = 2
): { sizeKb: number; feePerTx: number; totalFee: number } {
  const bytes = estimatedTxBytes(1, outputCount);
  const feePerTx = veriumFeeForSizeVrm(feeRatePerKb, bytes);
  return {
    sizeKb: bytes / 1000,
    feePerTx,
    totalFee: feePerTx * Math.max(1, transactionCount),
  };
}
