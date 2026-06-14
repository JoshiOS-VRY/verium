import { useSyncExternalStore } from 'react';

import type { CoinId } from '@/lib/coin/profile';
import type { ExplorerBlock } from '@/lib/explorer-api';
import { isPlaceholderTipHash } from '@/lib/tip-block-time';

/** Latest chain tip pushed from the local node watcher (`chain-tip-changed`). */
export interface ChainTip {
  coin: CoinId;
  height: number;
  hash: string;
  /** Unix seconds of the tip block, or 0 when block detail was unavailable. */
  time: number;
  block?: ExplorerBlock;
}

export interface ChainTipSnapshot {
  tip: ChainTip | null;
  /** Most recent blocks learned from the node, newest first. */
  recentBlocks: ExplorerBlock[];
}

const MAX_RECENT = 12;
const EMPTY: ChainTipSnapshot = { tip: null, recentBlocks: [] };

/** Coalesce burst tip events during catch-up sync (protects WebView from IPC storms). */
const TIP_NOTIFY_DEBOUNCE_MS = 500;

const snapshots = new Map<CoinId, ChainTipSnapshot>();
const listeners = new Map<CoinId, Set<() => void>>();
const pendingTips = new Map<CoinId, ChainTip>();
const tipNotifyTimers = new Map<CoinId, number>();

function getSnapshot(coin: CoinId): ChainTipSnapshot {
  return snapshots.get(coin) ?? EMPTY;
}

function notify(coin: CoinId): void {
  const set = listeners.get(coin);
  if (!set) return;
  for (const listener of set) listener();
}

function applyChainTip(tip: ChainTip): void {
  const prev = snapshots.get(tip.coin) ?? EMPTY;
  if (prev.tip?.hash === tip.hash) return;

  const blockTime =
    tip.time > 0 ? tip.time : isPlaceholderTipHash(tip.hash) ? 0 : Math.floor(Date.now() / 1000);

  const block: ExplorerBlock = tip.block ?? {
    id: tip.height,
    hash: tip.hash,
    height: tip.height,
    time: blockTime,
  };

  const blockHeight = Number(block.height);
  const recentBlocks = [block, ...prev.recentBlocks.filter((b) => Number(b.height) !== blockHeight)]
    .sort((a, b) => b.height - a.height)
    .slice(0, MAX_RECENT);

  snapshots.set(tip.coin, { tip: { ...tip, block }, recentBlocks });
  notify(tip.coin);
}

/** Record a new tip from the node watcher; ignores duplicate hashes. */
export function pushChainTip(tip: ChainTip): void {
  pendingTips.set(tip.coin, tip);
  if (tipNotifyTimers.has(tip.coin)) return;
  const timer = window.setTimeout(() => {
    tipNotifyTimers.delete(tip.coin);
    const latest = pendingTips.get(tip.coin);
    pendingTips.delete(tip.coin);
    if (latest) applyChainTip(latest);
  }, TIP_NOTIFY_DEBOUNCE_MS);
  tipNotifyTimers.set(tip.coin, timer);
}

export function subscribeChainTip(coin: CoinId, listener: () => void): () => void {
  let set = listeners.get(coin);
  if (!set) {
    set = new Set();
    listeners.set(coin, set);
  }
  set.add(listener);
  return () => {
    set?.delete(listener);
  };
}

/** Subscribe a React component to the latest tip + recent blocks for a coin. */
export function useChainTip(coin: CoinId): ChainTipSnapshot {
  return useSyncExternalStore(
    (cb) => subscribeChainTip(coin, cb),
    () => getSnapshot(coin)
  );
}
