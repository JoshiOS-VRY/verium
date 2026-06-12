/** In-app explorer routes (mobile light wallet). */
export function explorerTxPath(txid: string) {
  return `/explorer/tx/${encodeURIComponent(txid)}`;
}

export function explorerBlockPath(hashOrHeight: string | number) {
  return `/explorer/block/${encodeURIComponent(String(hashOrHeight))}`;
}

export function explorerAddressPath(address: string) {
  return `/explorer/address/${encodeURIComponent(address)}`;
}

export function isExplorerDetailPath(pathname: string) {
  return pathname.startsWith('/explorer/');
}
