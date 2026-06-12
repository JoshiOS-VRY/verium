import { describe, expect, it } from 'vitest';
import type { TransactionItem } from '@/lib/rpc/client';
import { filterTransactions, transactionMatchesHistoryFilter } from '@/lib/transaction-filters';

function tx(
  partial: Partial<TransactionItem> & Pick<TransactionItem, 'category'>
): TransactionItem {
  return {
    address: 'VRabc',
    amount: 1,
    confirmations: 10,
    txid: 'tx1',
    time: 1000,
    timereceived: 1000,
    ...partial,
    category: partial.category,
  };
}

describe('transaction-filters', () => {
  const rows = [
    tx({ category: 'receive', amount: 2, txid: 'a' }),
    tx({ category: 'send', amount: -1, txid: 'b' }),
    tx({ category: 'unconfirmed', confirmations: 0, txid: 'c' }),
    tx({ category: 'generate', txid: 'd' }),
  ];

  it('filters by category groups', () => {
    expect(transactionMatchesHistoryFilter(rows[0], 'receive')).toBe(true);
    expect(transactionMatchesHistoryFilter(rows[1], 'send')).toBe(true);
    expect(transactionMatchesHistoryFilter(rows[2], 'pending')).toBe(true);
    expect(transactionMatchesHistoryFilter(rows[3], 'mining')).toBe(true);
  });

  it('filters by search query', () => {
    const filtered = filterTransactions(rows, 'pending', '');
    expect(filtered).toHaveLength(1);
    expect(filtered[0].txid).toBe('c');
  });
});
