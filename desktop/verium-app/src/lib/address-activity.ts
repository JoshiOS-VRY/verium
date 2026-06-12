import type { IndexerAddressTransaction } from '@/lib/indexer-api';
import { parseIndexerAmountCoins } from '@/lib/indexer-amount';

export interface AddressActivityBar {
  id: string;
  label: string;
  delta: number;
  time?: number;
  isCoinbase?: boolean;
  isCoinstake?: boolean;
}

/** Recent tx deltas for bar charts (sorted oldest → newest). */
export function buildActivityBars(
  transactions: IndexerAddressTransaction[],
  max = 32
): AddressActivityBar[] {
  const sorted = [...transactions]
    .filter((tx) => tx.time != null || tx.netDelta)
    .sort((a, b) => (a.time ?? 0) - (b.time ?? 0))
    .slice(-max);

  return sorted.map((tx, index) => {
    const delta = parseIndexerAmountCoins(tx.netDelta);
    const when = tx.time ? new Date(tx.time * 1000) : null;
    const label = when
      ? when.toLocaleDateString(undefined, { month: 'short', day: 'numeric' })
      : `#${index + 1}`;

    return {
      id: tx.txid,
      label,
      delta,
      time: tx.time,
      isCoinbase: tx.isCoinbase,
      isCoinstake: tx.isCoinstake,
    };
  });
}

export interface AddressComposition {
  received: number;
  sent: number;
  balance: number;
  ticker: string;
}

export function buildAddressComposition(
  received: number,
  sent: number,
  balance: number,
  ticker: string
): AddressComposition {
  return {
    received: Math.max(0, received),
    sent: Math.max(0, sent),
    balance: Math.max(0, balance),
    ticker,
  };
}
