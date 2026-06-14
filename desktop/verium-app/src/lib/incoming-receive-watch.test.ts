import { describe, expect, it } from 'vitest';
import type { TransactionItem } from '@/lib/rpc/client';
import {
  receiveTimestampSec,
  shouldNotifyIncomingReceive,
} from '@/lib/incoming-receive-watch';

function tx(partial: Partial<TransactionItem> & Pick<TransactionItem, 'txid'>): TransactionItem {
  return {
    category: 'receive',
    amount: 1,
    confirmations: 1,
    time: 0,
    timereceived: 0,
    ...partial,
  };
}

describe('shouldNotifyIncomingReceive', () => {
  const watchStarted = 1_700_000_000;

  it('skips confirmed receives that mined before the watch session', () => {
    const old = tx({
      txid: 'old',
      confirmations: 6,
      blocktime: watchStarted - 3_600,
      time: watchStarted - 3_600,
      timereceived: watchStarted,
    });
    expect(shouldNotifyIncomingReceive(old, watchStarted)).toBe(false);
  });

  it('allows unconfirmed receives discovered during the session', () => {
    const pending = tx({
      txid: 'pending',
      category: 'unconfirmed',
      confirmations: 0,
      timereceived: watchStarted + 30,
    });
    expect(shouldNotifyIncomingReceive(pending, watchStarted)).toBe(true);
  });

  it('allows confirmed receives within the grace window', () => {
    const recent = tx({
      txid: 'recent',
      confirmations: 2,
      blocktime: watchStarted - 30,
      time: watchStarted - 30,
    });
    expect(shouldNotifyIncomingReceive(recent, watchStarted)).toBe(true);
  });
});

describe('receiveTimestampSec', () => {
  it('prefers block time for confirmed transactions', () => {
    const confirmed = tx({
      txid: 'a',
      confirmations: 3,
      blocktime: 100,
      timereceived: 999,
      time: 999,
    });
    expect(receiveTimestampSec(confirmed)).toBe(100);
  });
});
