import { useEffect, useRef } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useCoinWalletMode } from '@/hooks/useWalletMode';
import { useDaemonStatus } from '@/hooks/useDaemonStatus';
import { useUserPreferences } from '@/lib/user-preferences';
import { useWalletTransactions } from '@/hooks/useWalletTransactions';
import { isIncomingReceiveTx } from '@/lib/wallet-transactions-query';
import { coinQueryKey } from '@/lib/coin/profile';
import { rpcGetWalletInfo } from '@/lib/rpc/client';
import {
  INCOMING_RECEIVE_BATCH_DEBOUNCE_MS,
  incomingReceiveStorageKey,
  isIncomingReceiveBaselineOnly,
  loadSessionSeenReceiveTxids,
  markIncomingReceiveTxidsSeen,
  mergeIncomingReceiveEvent,
  persistSessionSeenReceiveTxids,
  shouldNotifyIncomingReceive,
} from '@/lib/incoming-receive-watch';

export type IncomingVrcEvent = import('@/lib/incoming-receive-watch').IncomingReceiveEvent;
export type IncomingVrcBatch = {
  events: IncomingVrcEvent[];
  totalAmount: number;
};

type IncomingVrcListener = (batch: IncomingVrcBatch) => void;

const listeners = new Set<IncomingVrcListener>();

export function subscribeIncomingVrc(listener: IncomingVrcListener): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

function emitIncomingVrc(batch: IncomingVrcBatch): void {
  if (batch.events.length === 0) return;
  for (const listener of listeners) {
    listener(batch);
  }
}

const SEEN_STORAGE_KEY = incomingReceiveStorageKey('vericoin');
const VERICOIN = 'vericoin' as const;

export function useIncomingVrcWatcher(): void {
  const notify = useUserPreferences((s) => s.prefs.notify_on_vrc_received !== false);
  const { isLight } = useCoinWalletMode(VERICOIN);
  const { data: status } = useDaemonStatus(VERICOIN, { enabled: notify && !isLight });
  const pollEnabled = notify && (isLight || status?.connected === true);
  const seen = useRef<Set<string>>(loadSessionSeenReceiveTxids(SEEN_STORAGE_KEY));
  const initialized = useRef(false);
  const watchStartedAtSec = useRef(Math.floor(Date.now() / 1000));
  const pending = useRef<IncomingVrcEvent[]>([]);
  const flushTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const walletInfo = useQuery({
    queryKey: coinQueryKey(VERICOIN, 'getwalletinfo'),
    queryFn: () => rpcGetWalletInfo(VERICOIN),
    enabled: pollEnabled,
    refetchInterval: false,
    staleTime: 5_000,
    gcTime: 30_000,
  });

  const txs = useWalletTransactions(VERICOIN, {
    enabled: pollEnabled,
    incomingWatch: pollEnabled && isLight,
  });

  useEffect(() => {
    return () => {
      if (flushTimer.current) clearTimeout(flushTimer.current);
    };
  }, []);

  useEffect(() => {
    if (!txs.isSuccess || txs.data === undefined) return;

    const incoming = txs.data.filter(isIncomingReceiveTx);
    const baselineOnly = isIncomingReceiveBaselineOnly(walletInfo.data, isLight);

    if (baselineOnly) {
      markIncomingReceiveTxidsSeen(
        seen.current,
        incoming.map((tx) => tx.txid),
        SEEN_STORAGE_KEY
      );
      return;
    }

    if (!initialized.current) {
      markIncomingReceiveTxidsSeen(
        seen.current,
        incoming.map((tx) => tx.txid),
        SEEN_STORAGE_KEY
      );
      watchStartedAtSec.current = Math.floor(Date.now() / 1000);
      initialized.current = true;
      return;
    }

    const scheduleFlush = () => {
      if (flushTimer.current) clearTimeout(flushTimer.current);
      flushTimer.current = setTimeout(() => {
        const events = pending.current;
        pending.current = [];
        flushTimer.current = null;
        if (events.length === 0) return;
        emitIncomingVrc({
          events,
          totalAmount: events.reduce((sum, e) => sum + e.amount, 0),
        });
      }, INCOMING_RECEIVE_BATCH_DEBOUNCE_MS);
    };

    const newlyDetected = new Map<string, IncomingVrcEvent>();
    let added = false;

    for (const tx of incoming) {
      if (seen.current.has(tx.txid)) continue;
      if (!shouldNotifyIncomingReceive(tx, watchStartedAtSec.current)) {
        seen.current.add(tx.txid);
        continue;
      }
      seen.current.add(tx.txid);
      mergeIncomingReceiveEvent(newlyDetected, tx);
      added = true;
    }

    if (!added) {
      if (seen.current.size > 0) {
        persistSessionSeenReceiveTxids(seen.current, SEEN_STORAGE_KEY);
      }
      return;
    }

    persistSessionSeenReceiveTxids(seen.current, SEEN_STORAGE_KEY);
    for (const event of newlyDetected.values()) {
      pending.current.push(event);
    }
    scheduleFlush();
  }, [txs.data, txs.isSuccess, walletInfo.data, isLight]);
}
