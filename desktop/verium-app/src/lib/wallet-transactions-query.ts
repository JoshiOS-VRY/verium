import { coinQueryKey, type CoinId } from '@/lib/coin/profile';
import { lightWalletRefreshPending } from '@/lib/light-wallet/client';
import { rpcListTransactions, type TransactionItem } from '@/lib/rpc/client';

/** Max rows for wallet tx poll (watchers + dashboard). Kept modest for memory. */
export const WALLET_TX_POLL_COUNT = 80;

/** Shared wallet transaction poll interval (all consumers dedupe on this key). */
export const WALLET_TX_POLL_INTERVAL_MS = 45_000;

/** Slower background poll when the window is hidden (notifications/chimes). */
export const WALLET_TX_BACKGROUND_POLL_MS = 120_000;

export function walletTransactionsQueryKey(coin: CoinId) {
  return coinQueryKey(coin, 'listtransactions', 'wallet');
}

/**
 * Prefix key covering every `listtransactions` query for a coin (the shared
 * "wallet" poll and the Transactions page "history" view). Invalidating this
 * refreshes both from one place so they don't each need an aggressive poll.
 */
export function walletTransactionsKeyPrefix(coin: CoinId) {
  return coinQueryKey(coin, 'listtransactions');
}

/** Receive rows for incoming-payment watchers (includes 0-conf). */
export function isIncomingReceiveTx(tx: TransactionItem): boolean {
  return (tx.category === 'receive' || tx.category === 'unconfirmed') && tx.amount > 0;
}

export async function fetchWalletTransactions(
  coin: CoinId,
  options?: { lightRefreshPending?: boolean }
) {
  if (options?.lightRefreshPending) {
    try {
      await lightWalletRefreshPending(coin);
    } catch {
      // Wallet locked or Electrum unreachable — still return cached history.
    }
  }
  return rpcListTransactions(coin, WALLET_TX_POLL_COUNT, 0);
}
