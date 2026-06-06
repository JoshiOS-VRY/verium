import { coinQueryKey, type CoinId } from "@/lib/coin/profile";
import { rpcListTransactions } from "@/lib/rpc/client";

/** Max rows for wallet tx poll (watchers + dashboard). Kept modest for memory. */
export const WALLET_TX_POLL_COUNT = 80;

/** Shared wallet transaction poll interval (all consumers dedupe on this key). */
export const WALLET_TX_POLL_INTERVAL_MS = 45_000;

/** Slower background poll when the window is hidden (notifications/chimes). */
export const WALLET_TX_BACKGROUND_POLL_MS = 120_000;

export function walletTransactionsQueryKey(coin: CoinId) {
  return coinQueryKey(coin, "listtransactions", "wallet");
}

export function fetchWalletTransactions(coin: CoinId) {
  return rpcListTransactions(coin, WALLET_TX_POLL_COUNT, 0);
}
