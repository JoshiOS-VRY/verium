import type { IndexerAmount } from '@/lib/indexer-api';

export function parseIndexerAmountCoins(amount: IndexerAmount | null | undefined): number {
  if (!amount?.amount) return 0;
  const n = Number.parseFloat(amount.amount.trim());
  return Number.isFinite(n) ? n : 0;
}

export function indexerTicker(amount: IndexerAmount | null | undefined): string {
  return amount?.ticker?.trim() || '';
}

export function sumIndexerAmountCoins(
  amounts: readonly (IndexerAmount | null | undefined)[]
): number {
  return amounts.reduce((sum, amount) => sum + parseIndexerAmountCoins(amount), 0);
}

export function formatIndexerCoinsTotal(total: number, ticker: string, digits = 8): string {
  if (!ticker) return total.toFixed(digits);
  const frac = Math.abs(total) >= 1 ? Math.min(4, digits) : Math.min(8, digits);
  return `${total.toFixed(frac)} ${ticker}`;
}

export function formatSignedIndexerAmount(amount: IndexerAmount | null | undefined): string {
  if (!amount) return '—';
  const coins = parseIndexerAmountCoins(amount);
  const ticker = indexerTicker(amount) || amount.ticker;
  const sign = coins > 0 ? '+' : coins < 0 ? '' : '';
  const frac = Math.abs(coins) >= 1 ? 4 : 8;
  return `${sign}${coins.toFixed(frac)} ${ticker}`;
}
