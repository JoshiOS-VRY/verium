/** Compact display for long base58 addresses on small screens. */
export function shortExplorerAddress(address: string, head = 10, tail = 8): string {
  const trimmed = address.trim();
  if (trimmed.length <= head + tail + 1) return trimmed;
  return `${trimmed.slice(0, head)}…${trimmed.slice(-tail)}`;
}

export function shortTxid(txid: string, head = 12, tail = 8): string {
  const trimmed = txid.trim();
  if (trimmed.length <= head + tail + 1) return trimmed;
  return `${trimmed.slice(0, head)}…${trimmed.slice(-tail)}`;
}
