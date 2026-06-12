import type { TransactionItem } from '@/lib/rpc/client';

export type TransactionHistoryFilter = 'all' | 'receive' | 'send' | 'pending' | 'mining';

export const TRANSACTION_HISTORY_FILTERS: {
  id: TransactionHistoryFilter;
  label: string;
}[] = [
  { id: 'all', label: 'All' },
  { id: 'receive', label: 'Received' },
  { id: 'send', label: 'Sent' },
  { id: 'pending', label: 'Pending' },
  { id: 'mining', label: 'Mining' },
];

const MINING_CATEGORIES = new Set(['generate', 'immature', 'stake', 'stake-mint', 'stake-orphan']);

export function transactionMatchesHistoryFilter(
  tx: TransactionItem,
  filter: TransactionHistoryFilter
): boolean {
  switch (filter) {
    case 'receive':
      return tx.category === 'receive' || tx.amount > 0;
    case 'send':
      return tx.category === 'send' || tx.amount < 0;
    case 'pending':
      return tx.category === 'unconfirmed' || tx.confirmations <= 0;
    case 'mining':
      return MINING_CATEGORIES.has(tx.category);
    default:
      return true;
  }
}

export function filterTransactions(
  txs: TransactionItem[],
  filter: TransactionHistoryFilter,
  searchQuery: string
): TransactionItem[] {
  const query = searchQuery.trim().toLowerCase();
  return txs.filter((tx) => {
    if (!transactionMatchesHistoryFilter(tx, filter)) return false;
    if (!query) return true;
    const haystack = `${tx.txid} ${tx.address ?? ''} ${tx.comment ?? ''}`.toLowerCase();
    return haystack.includes(query);
  });
}

export function countTransactionsByFilter(
  txs: TransactionItem[],
  filter: TransactionHistoryFilter
): number {
  if (filter === 'all') return txs.length;
  return txs.filter((tx) => transactionMatchesHistoryFilter(tx, filter)).length;
}
