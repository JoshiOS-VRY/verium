import { useActiveCoin, useCoinProfile } from '@/lib/coin/context';
import { coinQueryKey } from '@/lib/coin/profile';
import { useState, useEffect, useMemo } from 'react';
import { useLocation } from 'react-router-dom';
import { useMinerPayoutsQuery } from '@/hooks/usePoolQueries';
import { useLightWalletInstantReceiveSync } from '@/hooks/useLightWalletInstantReceiveSync';
import { useWalletMode } from '@/hooks/useWalletMode';
import { LIGHT_HISTORY_POLL_MS } from '@/lib/light-wallet/poll';
import { useWindowVisible } from '@/hooks/useWindowVisible';
import { resolvePoolDashboardAddress } from '@/lib/pool-dashboard-address';
import { useUserPreferences } from '@/lib/user-preferences';
import { rpcListAddressGroupings } from '@/lib/rpc/client';
import { useQuery } from '@tanstack/react-query';
import { ChevronLeft, ChevronRight, ArrowDownLeft, ArrowUpRight, Loader2 } from 'lucide-react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { ConfirmationProgress } from '@/components/ConfirmationProgress';
import { ExplorerLink } from '@/components/ExplorerLink';
import { ReceivePanel } from '@/components/ReceivePanel';
import { SendPanel } from '@/components/SendPanel';
import { TransactionHistoryCard } from '@/components/TransactionHistoryRow';
import { MobileBalanceHero } from '@/components/mobile/MobileBalanceHero';
import { MobileSegmented } from '@/components/mobile/MobileSegmented';
import { MobileTransactionHistory } from '@/components/mobile/MobileTransactionHistory';
import { WalletBalanceSummary } from '@/components/WalletBalanceSummary';
import { WalletUnlockGate } from '@/components/WalletUnlockGate';
import { rpcGetWalletInfo, rpcListTransactions, type TransactionItem } from '@/lib/rpc/client';
import {
  listTransactionsFetchParams,
  paginateTransactions,
  sortTransactionsNewestFirst,
  TRANSACTIONS_LIST_CAP,
  TRANSACTIONS_PAGE_SIZE,
  transactionPageCount,
} from '@/lib/transactions-list';
import { formatCoinAmount, formatTransactionTime } from '@/lib/units';
import {
  transactionCategoryLabel,
  transactionCategoryBadgeClass,
} from '@/lib/transaction-category';
import { cn, formatNumber } from '@/lib/utils';
import { consumePendingPaymentUri } from '@/lib/payment-uri-pending';

type TransferMode = 'send' | 'receive';
type MobileActivityView = TransferMode | 'history';

const stickyTableHeadClass =
  'sticky top-0 z-10 border-b border-border bg-bg-panel text-xs uppercase text-fg-subtle';
const stickyTableHeadCellClass = 'bg-bg-panel px-4 py-2 font-medium';

function TransferModeToggle({
  value,
  onChange,
}: {
  value: TransferMode;
  onChange: (mode: TransferMode) => void;
}) {
  return (
    <div
      role="radiogroup"
      aria-label="Send or receive"
      className="inline-flex rounded-md border border-border bg-bg-subtle p-1"
    >
      <button
        type="button"
        role="radio"
        aria-checked={value === 'send'}
        onClick={() => onChange('send')}
        className={cn(
          'inline-flex h-8 items-center gap-1.5 rounded px-3 text-xs font-medium transition-colors',
          value === 'send'
            ? 'bg-accent text-accent-fg'
            : 'text-fg-muted hover:bg-bg-panel hover:text-fg'
        )}
      >
        <ArrowUpRight className="h-3.5 w-3.5" />
        Send
      </button>
      <button
        type="button"
        role="radio"
        aria-checked={value === 'receive'}
        onClick={() => onChange('receive')}
        className={cn(
          'inline-flex h-8 items-center gap-1.5 rounded px-3 text-xs font-medium transition-colors',
          value === 'receive'
            ? 'bg-accent text-accent-fg'
            : 'text-fg-muted hover:bg-bg-panel hover:text-fg'
        )}
      >
        <ArrowDownLeft className="h-3.5 w-3.5" />
        Receive
      </button>
    </div>
  );
}

export function Transactions() {
  const coin = useActiveCoin();
  const profile = useCoinProfile();
  const location = useLocation();
  const { isLight, mobileOnly } = useWalletMode();
  const visible = useWindowVisible();
  const prefs = useUserPreferences((s) => s.prefs);
  const [mode, setMode] = useState<TransferMode>('send');
  const [mobileView, setMobileView] = useState<MobileActivityView>('send');
  const [prefill, setPrefill] = useState<{
    address?: string;
    amount?: string;
    label?: string;
  }>({});

  useLightWalletInstantReceiveSync(coin, isLight && visible);

  useEffect(() => {
    const pending = consumePendingPaymentUri();
    if (!pending) return;
    setMode('send');
    if (mobileOnly) setMobileView('send');
    setPrefill({
      address: pending.address,
      amount: pending.amount != null && pending.amount > 0 ? String(pending.amount) : undefined,
      label: pending.label ?? undefined,
    });
  }, [mobileOnly]);

  useEffect(() => {
    const view = (location.state as { mobileActivityView?: MobileActivityView })
      ?.mobileActivityView;
    if (!mobileOnly || !view) return;
    setMobileView(view);
    if (view === 'send' || view === 'receive') setMode(view);
  }, [location.state, mobileOnly]);
  const [page, setPage] = useState(0);

  useEffect(() => {
    setPage(0);
  }, [coin]);

  const wallet = useQuery({
    queryKey: coinQueryKey(coin, 'getwalletinfo'),
    queryFn: () => rpcGetWalletInfo(coin),
    refetchInterval: false,
  });

  const addressGroupings = useQuery({
    queryKey: coinQueryKey(coin, 'listaddressgroupings'),
    queryFn: () => rpcListAddressGroupings(coin),
    enabled: coin === 'verium' && !isLight,
    staleTime: 30_000,
  });

  const poolDashboardAddress = useMemo(
    () =>
      coin === 'verium' ? resolvePoolDashboardAddress(prefs, addressGroupings.data) : undefined,
    [coin, prefs, addressGroupings.data]
  );

  const poolPayouts = useMinerPayoutsQuery(poolDashboardAddress, coin === 'verium');

  const poolPayoutTxids = useMemo(() => {
    const rows = poolPayouts.data?.rows ?? [];
    return new Set(rows.map((p) => p.txid).filter((id): id is string => Boolean(id)));
  }, [poolPayouts.data]);

  const walletTxCount = wallet.data?.txcount ?? 0;
  // Light wallets used to report txcount=0 always, so never gate history fetches on it.
  const historyFetchCount = isLight
    ? TRANSACTIONS_LIST_CAP
    : listTransactionsFetchParams(walletTxCount).count;
  const historyFetchSkip = isLight ? 0 : listTransactionsFetchParams(walletTxCount).skip;
  const historyCapped = !isLight && walletTxCount > TRANSACTIONS_LIST_CAP;

  const lightSyncing = Boolean(wallet.data?.light_syncing);

  const txs = useQuery({
    queryKey: coinQueryKey(coin, 'listtransactions', 'history', isLight ? 'light' : walletTxCount),
    queryFn: async () => {
      if (historyFetchCount <= 0) return [];
      const rows = await rpcListTransactions(coin, historyFetchCount, historyFetchSkip);
      return sortTransactionsNewestFirst(rows);
    },
    enabled: wallet.isSuccess,
    // Light wallets: poll SQLite-backed history while the screen is open.
    refetchInterval:
      isLight && visible
        ? lightSyncing
          ? 5_000
          : LIGHT_HISTORY_POLL_MS
        : lightSyncing
          ? 20_000
          : false,
    retry: 1,
  });

  const sortedTxs = txs.data ?? [];
  const historyNeedsRefresh =
    isLight && sortedTxs.length > 0 && sortedTxs.every((t) => !t.time || t.time <= 0);
  const isHistoryLoading = wallet.isPending || (wallet.isSuccess && txs.isPending);
  const showEmptyHistory = !isHistoryLoading && !txs.isError && sortedTxs.length === 0;

  const totalPages = transactionPageCount(sortedTxs.length);
  const effectivePage = Math.min(page, Math.max(0, totalPages - 1));
  const pageRows = useMemo(
    () => paginateTransactions(sortedTxs, effectivePage),
    [sortedTxs, effectivePage]
  );
  const rangeFrom = sortedTxs.length === 0 ? 0 : effectivePage * TRANSACTIONS_PAGE_SIZE + 1;
  const rangeTo = Math.min(sortedTxs.length, (effectivePage + 1) * TRANSACTIONS_PAGE_SIZE);

  function renderPagination({
    totalItems,
    totalPages: pages,
    rangeFrom: from,
    rangeTo: to,
    cappedNote,
  }: {
    totalItems: number;
    totalPages: number;
    rangeFrom: number;
    rangeTo: number;
    cappedNote?: string;
  }) {
    if (totalItems === 0) return null;
    return (
      <div className="flex flex-col gap-2 border-t border-border px-4 py-3 sm:flex-row sm:items-center sm:justify-between">
        <div className="text-xs text-fg-muted">
          <span>
            Showing {formatNumber(from)}–{formatNumber(to)} of {formatNumber(totalItems)}
          </span>
          {cappedNote ? <span className="mt-1 block text-fg-subtle">{cappedNote}</span> : null}
        </div>
        <div className="flex flex-wrap items-center justify-center gap-2 sm:justify-end">
          <Button
            type="button"
            variant="secondary"
            size="sm"
            disabled={effectivePage <= 0}
            onClick={() => setPage((p) => Math.max(0, p - 1))}
            aria-label="Previous page"
          >
            <ChevronLeft className="h-3.5 w-3.5" />
            Previous
          </Button>
          <span className="text-center text-xs tabular-nums text-fg-muted">
            Page {effectivePage + 1} of {pages}
          </span>
          <Button
            type="button"
            variant="secondary"
            size="sm"
            disabled={effectivePage >= pages - 1}
            onClick={() => setPage((p) => p + 1)}
            aria-label="Next page"
          >
            Next
            <ChevronRight className="h-3.5 w-3.5" />
          </Button>
        </div>
      </div>
    );
  }

  const historySection = (
    <Card className={mobileOnly ? 'mobile-panel overflow-hidden rounded-2xl' : undefined}>
      <CardHeader className={mobileOnly ? 'px-4 py-4' : undefined}>
        <CardTitle className="flex items-center gap-2 text-base">
          {mobileOnly ? 'History' : 'Recent transactions'}
          {isHistoryLoading ? (
            <Loader2 className="h-4 w-4 animate-spin text-accent" aria-hidden />
          ) : null}
        </CardTitle>
        <CardDescription>
          {isHistoryLoading
            ? 'Loading your wallet transaction history…'
            : txs.isError
              ? 'Could not load transaction history from the wallet.'
              : showEmptyHistory
                ? 'Transactions you send or receive will appear here.'
                : historyNeedsRefresh
                  ? 'Loading amounts and dates from the explorer index…'
                  : 'Newest first.'}
        </CardDescription>
      </CardHeader>
      <CardContent className="min-w-0 p-0">
        <div
          className={cn(
            'max-h-[480px] min-w-0 overflow-x-hidden',
            mobileOnly ? 'overflow-y-auto px-3 py-3' : 'overflow-auto'
          )}
        >
          {isHistoryLoading ? (
            mobileOnly ? (
              <div className="flex flex-col gap-2.5">
                {Array.from({ length: 6 }).map((_, i) => (
                  <div
                    key={`loading-card-${i}`}
                    className="h-24 animate-pulse rounded-xl border border-border bg-bg-subtle"
                  />
                ))}
              </div>
            ) : (
              <table className="w-full border-collapse text-sm">
                <thead className={stickyTableHeadClass}>
                  <tr>
                    <th className={cn(stickyTableHeadCellClass, 'text-left')}>When</th>
                    <th className={cn(stickyTableHeadCellClass, 'text-left')}>Type</th>
                    <th className={cn(stickyTableHeadCellClass, 'text-left')}>Address</th>
                    <th className={cn(stickyTableHeadCellClass, 'text-right')}>Amount</th>
                    <th className={cn(stickyTableHeadCellClass, 'text-right')}>Confs</th>
                    <th className={cn(stickyTableHeadCellClass, 'text-right')}>Explorer</th>
                  </tr>
                </thead>
                <tbody>
                  {Array.from({ length: 6 }).map((_, i) => (
                    <tr key={`loading-${i}`} className="border-t border-border">
                      <td colSpan={6} className="px-4 py-2">
                        <div className="h-4 animate-pulse rounded bg-bg-subtle" />
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )
          ) : showEmptyHistory ? (
            <div className={cn('py-10', mobileOnly ? 'px-2' : 'px-4')}>
              <div className="mx-auto max-w-md space-y-5 text-center">
                {!mobileOnly && (
                  <div className="space-y-2.5 opacity-50" aria-hidden>
                    {Array.from({ length: 4 }).map((_, i) => (
                      <div key={`empty-skeleton-${i}`} className="flex items-center gap-3">
                        <div className="h-3 w-24 shrink-0 animate-pulse rounded bg-bg-subtle" />
                        <div className="h-3 w-16 shrink-0 animate-pulse rounded bg-bg-subtle" />
                        <div className="h-3 min-w-0 flex-1 animate-pulse rounded bg-bg-subtle" />
                        <div className="h-3 w-14 shrink-0 animate-pulse rounded bg-bg-subtle" />
                      </div>
                    ))}
                  </div>
                )}
                <div className="space-y-1.5">
                  <p className="text-sm font-medium text-fg-muted">
                    {lightSyncing
                      ? 'Scanning for past transactions…'
                      : mobileOnly
                        ? 'No transactions yet'
                        : 'This wallet has not made any transactions yet.'}
                  </p>
                  <p className="text-xs text-fg-subtle">
                    {lightSyncing
                      ? `After import, the light wallet scans your addresses on Vericonomy servers. History can take a few minutes — pull to refresh on Dashboard or wait here.`
                      : mobileOnly
                        ? `Send or receive ${profile.symbol} and your activity will show up here. Imported wallets only show on-chain history once address scan finds your past receives.`
                        : `Use Send or Receive above to move ${profile.symbol}. Your history will show up here once activity is recorded in the wallet.`}
                  </p>
                </div>
              </div>
            </div>
          ) : mobileOnly ? (
            <div className="flex flex-col gap-2.5">
              {pageRows.map((tx: TransactionItem) => (
                <TransactionHistoryCard
                  key={`${tx.txid}-${tx.category}-${tx.address ?? ''}-${tx.time}`}
                  tx={tx}
                  coin={coin}
                  isPoolPayout={coin === 'verium' && poolPayoutTxids.has(tx.txid)}
                />
              ))}
            </div>
          ) : (
            <table className="w-full border-collapse text-sm">
              <thead className={stickyTableHeadClass}>
                <tr>
                  <th className={cn(stickyTableHeadCellClass, 'text-left')}>When</th>
                  <th className={cn(stickyTableHeadCellClass, 'text-left')}>Type</th>
                  <th className={cn(stickyTableHeadCellClass, 'text-left')}>Address</th>
                  <th className={cn(stickyTableHeadCellClass, 'text-right')}>Amount</th>
                  <th className={cn(stickyTableHeadCellClass, 'text-right')}>Confs</th>
                  <th className={cn(stickyTableHeadCellClass, 'text-right')}>Explorer</th>
                </tr>
              </thead>
              <tbody>
                {pageRows.map((tx: TransactionItem) => (
                  <tr
                    key={`${tx.txid}-${tx.category}-${tx.address ?? ''}-${tx.time}`}
                    className="border-t border-border odd:bg-bg-subtle/30"
                  >
                    <td className="px-4 py-2 text-xs text-fg-muted">
                      {formatTransactionTime(tx.time)}
                    </td>
                    <td className="px-4 py-2">
                      <div className="flex flex-wrap items-center gap-1.5">
                        <Badge className={transactionCategoryBadgeClass(tx.category)}>
                          {transactionCategoryLabel(tx.category)}
                        </Badge>
                        {coin === 'verium' && poolPayoutTxids.has(tx.txid) ? (
                          <Badge tone="neutral">Pool payout</Badge>
                        ) : null}
                      </div>
                    </td>
                    <td className="truncate px-4 py-2 text-xs">{tx.address ?? '—'}</td>
                    <td className="px-4 py-2 text-right tabular-nums">
                      {formatCoinAmount(tx.amount, coin, 8)}
                    </td>
                    <td className="px-4 py-2 text-right">
                      <ConfirmationProgress
                        confirmations={tx.confirmations}
                        category={tx.category}
                      />
                    </td>
                    <td className="px-4 py-2 text-right">
                      <ExplorerLink
                        coin={coin}
                        target={{ kind: 'tx', txid: tx.txid }}
                        label="View"
                        title={`Open tx ${tx.txid} on the explorer`}
                      />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
        {!isHistoryLoading && !showEmptyHistory && !txs.isError
          ? renderPagination({
              totalItems: sortedTxs.length,
              totalPages,
              rangeFrom,
              rangeTo,
              cappedNote: historyCapped
                ? `Showing the ${formatNumber(TRANSACTIONS_LIST_CAP)} most recent wallet entries.`
                : undefined,
            })
          : null}
      </CardContent>
    </Card>
  );

  return (
    <WalletUnlockGate
      title="Unlock to send and view transactions"
      description={`Enter your wallet passphrase to send or receive ${profile.symbol} and view your transaction history.`}
    >
      {mobileOnly ? (
        <div className="mobile-page">
          {mobileView === 'history' ? <MobileBalanceHero /> : null}
          <MobileSegmented
            value={mobileView}
            ariaLabel="Activity"
            onChange={(view) => {
              setMobileView(view);
              if (view === 'send' || view === 'receive') setMode(view);
            }}
            options={[
              { value: 'send', label: 'Send', icon: <ArrowUpRight className="h-3.5 w-3.5" /> },
              {
                value: 'receive',
                label: 'Receive',
                icon: <ArrowDownLeft className="h-3.5 w-3.5" />,
              },
              { value: 'history', label: 'History' },
            ]}
          />
          {mobileView === 'send' ? (
            <section className="mobile-panel min-w-0">
              <div className="mb-3 px-1">
                <h2 className="text-lg font-semibold tracking-tight text-fg">
                  Send {profile.symbol}
                </h2>
                <p className="mt-0.5 text-sm text-fg-muted">
                  Scan, paste, or enter a {profile.displayName} address.
                </p>
              </div>
              <SendPanel
                initialAddress={prefill.address}
                initialAmount={prefill.amount}
                initialLabel={prefill.label}
              />
            </section>
          ) : null}
          {mobileView === 'receive' ? (
            <section className="mobile-panel min-w-0">
              <div className="mb-3 px-1">
                <h2 className="text-lg font-semibold tracking-tight text-fg">
                  Receive {profile.symbol}
                </h2>
                <p className="mt-0.5 text-sm text-fg-muted">
                  Show a QR code for in-person payments.
                </p>
              </div>
              <ReceivePanel />
            </section>
          ) : null}
          {mobileView === 'history' ? (
            <MobileTransactionHistory
              coin={coin}
              txs={sortedTxs}
              isLoading={isHistoryLoading}
              isError={txs.isError}
              showEmpty={showEmptyHistory}
              lightSyncing={lightSyncing}
              historyNeedsRefresh={historyNeedsRefresh}
              poolPayoutTxids={poolPayoutTxids}
              profileSymbol={profile.symbol}
            />
          ) : null}
        </div>
      ) : (
        <div className="flex min-w-0 max-w-full flex-col gap-6">
          <WalletBalanceSummary />

          <Card>
            <CardHeader className="flex-row flex-wrap items-start justify-between gap-4">
              <div>
                <CardTitle>{mode === 'send' ? 'Send' : 'Receive'}</CardTitle>
                <CardDescription>
                  {mode === 'send'
                    ? `Pay to one or more ${profile.displayName} addresses. Labels are saved locally with the transaction comment.`
                    : `Create ${profile.symbol} receiving addresses with optional label, amount, and message.`}
                </CardDescription>
              </div>
              <TransferModeToggle value={mode} onChange={setMode} />
            </CardHeader>
            <CardContent>
              {mode === 'send' ? (
                <SendPanel
                  initialAddress={prefill.address}
                  initialAmount={prefill.amount}
                  initialLabel={prefill.label}
                />
              ) : (
                <ReceivePanel />
              )}
            </CardContent>
          </Card>

          {historySection}
        </div>
      )}
    </WalletUnlockGate>
  );
}
