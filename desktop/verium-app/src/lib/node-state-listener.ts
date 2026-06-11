/**
 * Singleton Tauri `node-state-changed` listener per coin.
 *
 * Without this, every `useNodeStatus()` mount registers its own listener (~8+
 * concurrent handlers on the dashboard). One channel per coin invalidates all
 * subscribers instead.
 */

import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { CoinId } from '@/lib/coin/profile';

type CoinChannel = {
  subscribers: Set<() => void>;
  unlisten: UnlistenFn | null;
  listenPromise: Promise<UnlistenFn> | null;
};

const channels = new Map<string, CoinChannel>();

function listenerCount(): number {
  let n = 0;
  for (const ch of channels.values()) n += ch.subscribers.size;
  return n;
}

async function reportListenerCount(): Promise<void> {
  if (!import.meta.env.DEV) return;
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('set_node_state_listener_count', { count: listenerCount() });
  } catch {
    /* telemetry optional */
  }
}

function getChannel(coin: CoinId): CoinChannel {
  let ch = channels.get(coin);
  if (!ch) {
    ch = { subscribers: new Set(), unlisten: null, listenPromise: null };
    channels.set(coin, ch);
    ch.listenPromise = listen<{ coin: string }>('node-state-changed', (event) => {
      if (event.payload.coin !== coin) return;
      const current = channels.get(coin);
      if (!current) return;
      for (const fn of current.subscribers) fn();
    }).then((unlisten) => {
      const current = channels.get(coin);
      if (current) current.unlisten = unlisten;
      return unlisten;
    });
  }
  return ch;
}

/** Subscribe to node state changes for a coin. Returns an unsubscribe function. */
export function subscribeNodeStateChanged(coin: CoinId, onChange: () => void): () => void {
  const ch = getChannel(coin);
  ch.subscribers.add(onChange);
  void reportListenerCount();

  return () => {
    ch.subscribers.delete(onChange);
    if (ch.subscribers.size === 0) {
      if (ch.unlisten) void ch.unlisten();
      else if (ch.listenPromise) void ch.listenPromise.then((u) => u());
      channels.delete(coin);
    }
    void reportListenerCount();
  };
}

/** Dev-only: number of active coin channels. */
export function nodeStateChannelCount(): number {
  return channels.size;
}
