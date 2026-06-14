import type { TransactionItem, WalletInfo } from '@/lib/rpc/client';
import { addSeenTxid } from '@/lib/seen-txid-set';

export const INCOMING_RECEIVE_BATCH_DEBOUNCE_MS = 800;
export const MAX_SEEN_RECEIVE_TXIDS = 2_000;

/** Allow small clock skew and same-block timing vs JS session start. */
const NOTIFY_GRACE_SEC = 120;

export interface IncomingReceiveEvent {
  txid: string;
  amount: number;
  address?: string;
  confirmations: number;
  time?: number;
  blockheight?: number;
}

export function incomingReceiveStorageKey(coin: 'verium' | 'vericoin'): string {
  return coin === 'verium'
    ? 'verium-notified-receive-txids'
    : 'verium-notified-vrc-receive-txids';
}

function sessionStore(): Storage | null {
  if (typeof sessionStorage !== 'undefined') return sessionStorage;
  return null;
}

export function loadSessionSeenReceiveTxids(storageKey: string): Set<string> {
  const store = sessionStore();
  if (!store) return new Set();
  try {
    const raw = store.getItem(storageKey);
    if (!raw) return new Set();
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) return new Set();
    return new Set(parsed.filter((id): id is string => typeof id === 'string'));
  } catch {
    return new Set();
  }
}

export function persistSessionSeenReceiveTxids(seen: Set<string>, storageKey: string): void {
  const store = sessionStore();
  if (!store) return;
  try {
    const ids = [...seen];
    const trimmed =
      ids.length > MAX_SEEN_RECEIVE_TXIDS ? ids.slice(-MAX_SEEN_RECEIVE_TXIDS) : ids;
    store.setItem(storageKey, JSON.stringify(trimmed));
  } catch {
    // Ignore quota / privacy mode failures.
  }
}

/** Best-effort on-chain / mempool time for "did this happen while the app was open?" */
export function receiveTimestampSec(tx: TransactionItem): number {
  if (tx.confirmations <= 0) {
    return tx.timereceived || tx.time || tx.blocktime || 0;
  }
  return tx.blocktime || tx.time || tx.timereceived || 0;
}

export function shouldNotifyIncomingReceive(
  tx: TransactionItem,
  watchStartedAtSec: number
): boolean {
  const ts = receiveTimestampSec(tx);
  if (ts > 0) {
    return ts >= watchStartedAtSec - NOTIFY_GRACE_SEC;
  }
  // Unknown time — only treat unconfirmed mempool receives as live.
  return tx.confirmations <= 0;
}

export function isWalletReadyToWatchIncoming(
  walletInfo: WalletInfo | null | undefined,
  isLight: boolean
): boolean {
  if (!walletInfo) return false;
  if (typeof walletInfo.scanning === 'object' && walletInfo.scanning) return false;
  if (walletInfo.light_syncing || walletInfo.light_setup_syncing) return false;
  if (walletInfo.light_scan_phase && walletInfo.light_scan_phase !== 'complete') return false;
  if (isLight && walletInfo.private_keys_enabled !== true) return false;
  return true;
}

/** True while history may still be backfilling — mark seen but never toast. */
export function isIncomingReceiveBaselineOnly(
  walletInfo: WalletInfo | null | undefined,
  isLight: boolean
): boolean {
  return !isWalletReadyToWatchIncoming(walletInfo, isLight);
}

export function mergeIncomingReceiveEvent(
  map: Map<string, IncomingReceiveEvent>,
  tx: TransactionItem
): void {
  const existing = map.get(tx.txid);
  if (existing) {
    existing.amount += tx.amount;
    if (!existing.address && tx.address) existing.address = tx.address;
    existing.confirmations = Math.max(existing.confirmations, tx.confirmations);
    if (!existing.time && tx.time) existing.time = tx.time;
    if (!existing.blockheight && tx.blockheight) existing.blockheight = tx.blockheight;
    return;
  }

  map.set(tx.txid, {
    txid: tx.txid,
    amount: tx.amount,
    address: tx.address,
    confirmations: tx.confirmations,
    time: tx.time,
    blockheight: tx.blockheight,
  });
}

export function toIncomingReceiveEvent(tx: TransactionItem): IncomingReceiveEvent {
  return {
    txid: tx.txid,
    amount: tx.amount,
    address: tx.address,
    confirmations: tx.confirmations,
    time: tx.time,
    blockheight: tx.blockheight,
  };
}

export function markIncomingReceiveTxidsSeen(
  seen: Set<string>,
  txids: Iterable<string>,
  storageKey: string
): void {
  for (const txid of txids) {
    addSeenTxid(seen, txid, MAX_SEEN_RECEIVE_TXIDS);
  }
  persistSessionSeenReceiveTxids(seen, storageKey);
}
