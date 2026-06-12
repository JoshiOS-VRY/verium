import { invoke } from '@tauri-apps/api/core';
import type { CoinId } from '@/lib/coin/profile';

export interface IndexerAmount {
  amount: string;
  ticker: string;
  decimalPlaces?: number;
}

export interface IndexerTransactionSummary {
  txid: string;
  blockHeight?: number;
  blockHash?: string;
  txIndex?: number;
  time?: number;
  isCoinbase?: boolean;
  isCoinstake?: boolean;
}

export interface IndexerVin {
  n: number;
  prevTxid?: string | null;
  prevVout?: number | null;
  address?: string | null;
  value?: IndexerAmount | null;
  resolved?: boolean;
}

export interface IndexerVout {
  n: number;
  address?: string | null;
  value?: IndexerAmount | null;
  isSpent?: boolean;
}

export interface IndexerAddressEvent {
  address: string;
  delta?: IndexerAmount | null;
  eventType?: string | null;
}

export interface IndexerTransactionDetail {
  chainId?: string;
  txid: string;
  found: boolean;
  trusted?: boolean;
  transaction?: IndexerTransactionSummary | null;
  inputs?: IndexerVin[];
  outputs?: IndexerVout[];
  addressEvents?: IndexerAddressEvent[];
}

export interface IndexerBlockSummary {
  height: number;
  hash: string;
  previousHash?: string | null;
  nextHash?: string | null;
  time?: number;
  txCount?: number;
  size?: number;
  difficulty?: string | null;
}

export interface IndexerPaging {
  limit: number;
  offset: number;
  total: number;
  hasMore?: boolean;
}

export interface IndexerBlockDetail {
  chainId?: string;
  found: boolean;
  trusted?: boolean;
  block?: IndexerBlockSummary | null;
  paging?: IndexerPaging | null;
  transactions?: IndexerTransactionSummary[];
}

export interface IndexerAddressBalance {
  address: string;
  balanceAtomic?: string | null;
  balance?: IndexerAmount | null;
  totalReceivedAtomic?: string | null;
  totalReceived?: IndexerAmount | null;
  totalSentAtomic?: string | null;
  totalSent?: IndexerAmount | null;
  txCount?: number;
  lastSeenHeight?: number | null;
  firstSeenHeight?: number | null;
  firstSeenTime?: number | null;
}

export interface IndexerAddressRichlist {
  enabled?: boolean;
  eligible?: boolean | null;
  rank?: number | null;
  total?: number | null;
  percentile?: number | null;
}

export interface IndexerAddressTransaction {
  txid: string;
  blockHeight?: number;
  blockHash?: string;
  txIndex?: number;
  time?: number;
  netDeltaAtomic?: string | null;
  netDelta?: IndexerAmount | null;
  isCoinbase?: boolean;
  isCoinstake?: boolean;
}

export interface CumulativeBalancePoint {
  time?: number;
  blockHeight?: number;
  balance: IndexerAmount;
}

export interface CumulativeBalanceSeries {
  points: CumulativeBalancePoint[];
  currentBalance: IndexerAmount;
  txCountUsed: number;
  txCountTotal?: number;
  complete: boolean;
}

export interface IndexerAddressDetail {
  chainId?: string;
  address: string;
  found: boolean;
  trusted?: boolean;
  source?: {
    label?: string;
    trustLevel?: string;
    healthStatus?: string;
    message?: string;
  };
  balance?: IndexerAddressBalance | null;
  richlist?: IndexerAddressRichlist | null;
  paging?: IndexerPaging | null;
  transactions?: IndexerAddressTransaction[];
}

export function formatIndexerAmount(amount: IndexerAmount | null | undefined): string {
  if (!amount) return '—';
  return `${amount.amount} ${amount.ticker}`;
}

export function fetchIndexerTransaction(
  coin: CoinId,
  txid: string
): Promise<IndexerTransactionDetail> {
  return invoke<IndexerTransactionDetail>('fetch_indexer_transaction', { coin, txid });
}

export function fetchIndexerBlock(
  coin: CoinId,
  hashOrHeight: string,
  limit = 25,
  offset = 0
): Promise<IndexerBlockDetail> {
  return invoke<IndexerBlockDetail>('fetch_indexer_block', {
    coin,
    hashOrHeight,
    limit,
    offset,
  });
}

export function fetchIndexerAddress(
  coin: CoinId,
  address: string,
  limit = 25,
  offset = 0
): Promise<IndexerAddressDetail> {
  return invoke<IndexerAddressDetail>('fetch_indexer_address', {
    coin,
    address,
    limit,
    offset,
  });
}

export function fetchIndexerAddressCumulativeSeries(
  coin: CoinId,
  address: string,
  maxTxs?: number
): Promise<CumulativeBalanceSeries> {
  return invoke<CumulativeBalanceSeries>('fetch_indexer_address_cumulative_series', {
    coin,
    address,
    maxTxs,
  });
}
