import { useMemo, useState } from 'react';
import { Search, X } from 'lucide-react';
import { TransactionHistoryCard } from '@/components/TransactionHistoryRow';
import type { CoinId } from '@/lib/coin/profile';
import type { TransactionItem } from '@/lib/rpc/client';
import {
  TRANSACTION_HISTORY_FILTERS,
  countTransactionsByFilter,
  filterTransactions,
  type TransactionHistoryFilter,
} from '@/lib/transaction-filters';
import { TRANSACTIONS_PAGE_SIZE } from '@/lib/transactions-list';
import { cn } from '@/lib/utils';

const MOBILE_HISTORY_INITIAL = TRANSACTIONS_PAGE_SIZE;
const MOBILE_HISTORY_STEP = 25;

export function MobileTransactionHistory({
  coin,
  txs,
  isLoading,
  isError,
  showEmpty,
  lightSyncing,
  historyNeedsRefresh,
  poolPayoutTxids,
  profileSymbol,
}: {
  coin: CoinId;
  txs: TransactionItem[];
  isLoading: boolean;
  isError: boolean;
  showEmpty: boolean;
  lightSyncing: boolean;
  historyNeedsRefresh: boolean;
  poolPayoutTxids: Set<string>;
  profileSymbol: string;
}) {
  const [filter, setFilter] = useState<TransactionHistoryFilter>('all');
  const [search, setSearch] = useState('');
  const [visibleCount, setVisibleCount] = useState(MOBILE_HISTORY_INITIAL);

  const filtered = useMemo(() => filterTransactions(txs, filter, search), [txs, filter, search]);

  const visibleRows = filtered.slice(0, visibleCount);
  const hasMore = filtered.length > visibleCount;

  const filterCounts = useMemo(
    () =>
      TRANSACTION_HISTORY_FILTERS.map((f) => ({
        ...f,
        count: countTransactionsByFilter(txs, f.id),
      })),
    [txs]
  );

  const statusLine = isLoading
    ? 'Loading your wallet transaction history…'
    : isError
      ? 'Could not load transaction history from the wallet.'
      : showEmpty
        ? 'Transactions you send or receive will appear here.'
        : historyNeedsRefresh
          ? 'Loading amounts and dates from the explorer index…'
          : filtered.length === 0
            ? 'No transactions match your filters.'
            : `${filtered.length} transaction${filtered.length === 1 ? '' : 's'}`;

  return (
    <section className="mobile-panel min-w-0">
      <div className="mb-3 px-1">
        <h2 className="text-lg font-semibold tracking-tight text-fg">History</h2>
        <p className="mt-0.5 text-sm text-fg-muted">{statusLine}</p>
      </div>

      {!isLoading && !showEmpty && !isError && (
        <div className="mb-3 space-y-3">
          <div className="relative">
            <Search
              className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-fg-subtle"
              aria-hidden
            />
            <input
              type="search"
              value={search}
              onChange={(e) => {
                setSearch(e.target.value);
                setVisibleCount(MOBILE_HISTORY_INITIAL);
              }}
              placeholder="Search address or transaction ID"
              enterKeyHint="search"
              autoCapitalize="off"
              autoCorrect="off"
              className="mobile-input w-full pl-10 pr-10"
            />
            {search && (
              <button
                type="button"
                onClick={() => setSearch('')}
                className="absolute right-2 top-1/2 flex h-9 w-9 -translate-y-1/2 items-center justify-center rounded-lg text-fg-muted active:bg-bg-subtle"
                aria-label="Clear search"
              >
                <X className="h-4 w-4" />
              </button>
            )}
          </div>

          <div
            className="mobile-transaction-filters flex gap-2 overflow-x-auto pb-0.5 [-ms-overflow-style:none] [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
            role="tablist"
            aria-label="Transaction filters"
          >
            {filterCounts.map((f) => {
              const active = filter === f.id;
              return (
                <button
                  key={f.id}
                  type="button"
                  role="tab"
                  aria-selected={active}
                  onClick={() => {
                    setFilter(f.id);
                    setVisibleCount(MOBILE_HISTORY_INITIAL);
                  }}
                  className={cn(
                    'flex shrink-0 items-center gap-1.5 rounded-full border px-3 py-2 text-xs font-semibold transition-colors touch-manipulation',
                    active
                      ? 'border-accent bg-accent text-accent-fg'
                      : 'border-border bg-bg-subtle text-fg-muted active:bg-bg-panel'
                  )}
                >
                  {f.label}
                  <span
                    className={cn(
                      'rounded-full px-1.5 py-0.5 text-[10px] tabular-nums',
                      active ? 'bg-accent-fg/20 text-accent-fg' : 'bg-bg-panel text-fg-subtle'
                    )}
                  >
                    {f.count}
                  </span>
                </button>
              );
            })}
          </div>
        </div>
      )}

      <div className="min-w-0">
        {isLoading ? (
          <div className="flex flex-col gap-2.5">
            {Array.from({ length: 6 }).map((_, i) => (
              <div
                key={`loading-card-${i}`}
                className="h-24 animate-pulse rounded-xl border border-border bg-bg-subtle"
              />
            ))}
          </div>
        ) : showEmpty ? (
          <div className="rounded-2xl border border-border bg-bg-panel/60 px-4 py-10 text-center">
            <p className="text-sm font-medium text-fg-muted">
              {lightSyncing ? 'Scanning for past transactions…' : 'No transactions yet'}
            </p>
            <p className="mt-2 text-xs text-fg-subtle">
              {lightSyncing
                ? 'Imported wallets can take a few minutes to load history from the index.'
                : `Send or receive ${profileSymbol} and your activity will show up here.`}
            </p>
          </div>
        ) : isError ? (
          <div className="rounded-2xl border border-danger/30 bg-danger/10 px-4 py-6 text-center text-sm text-danger">
            Could not load transaction history.
          </div>
        ) : filtered.length === 0 ? (
          <div className="rounded-2xl border border-border bg-bg-panel/60 px-4 py-10 text-center">
            <p className="text-sm font-medium text-fg-muted">No matching transactions</p>
            <p className="mt-2 text-xs text-fg-subtle">Try another filter or clear your search.</p>
          </div>
        ) : (
          <>
            <div className="flex flex-col gap-2.5">
              {visibleRows.map((tx) => (
                <TransactionHistoryCard
                  key={`${tx.txid}-${tx.category}-${tx.address ?? ''}-${tx.time}`}
                  tx={tx}
                  coin={coin}
                  isPoolPayout={coin === 'verium' && poolPayoutTxids.has(tx.txid)}
                />
              ))}
            </div>
            {hasMore && (
              <button
                type="button"
                onClick={() => setVisibleCount((n) => n + MOBILE_HISTORY_STEP)}
                className="mt-3 flex min-h-[44px] w-full items-center justify-center rounded-xl border border-border bg-bg-subtle/60 text-sm font-medium text-fg active:bg-bg-subtle"
              >
                Load more ({filtered.length - visibleCount} remaining)
              </button>
            )}
            {isLoading ? null : (
              <p className="mt-3 text-center text-[10px] text-fg-subtle">
                Showing {visibleRows.length} of {filtered.length}
                {filter !== 'all' || search ? ` (from ${txs.length} total)` : ''}
              </p>
            )}
          </>
        )}
      </div>
    </section>
  );
}
