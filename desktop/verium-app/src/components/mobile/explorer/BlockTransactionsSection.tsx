import { useState } from 'react';
import { ChevronDown, Hammer, Layers } from 'lucide-react';
import { ExplorerCardLink } from '@/components/ExplorerLink';
import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import type { IndexerPaging, IndexerTransactionSummary } from '@/lib/indexer-api';
import { formatTransactionTime } from '@/lib/units';
import { cn } from '@/lib/utils';
import { shortTxid } from './tx-detail-utils';

function BlockTxRow({ tx, index }: { tx: IndexerTransactionSummary; index: number }) {
  const position = tx.txIndex ?? index;

  return (
    <ExplorerCardLink
      target={{ kind: 'tx', txid: tx.txid }}
      ariaLabel={`View transaction ${tx.txid}`}
      className="rounded-xl border border-border/70 bg-bg-subtle/30 px-3 py-2.5"
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-1.5">
            <Badge tone="neutral" className="text-[10px] tabular-nums">
              #{position + 1}
            </Badge>
            {tx.isCoinbase ? (
              <Badge tone="warning" className="gap-0.5 text-[10px]">
                <Hammer className="h-3 w-3" aria-hidden />
                Coinbase
              </Badge>
            ) : tx.isCoinstake ? (
              <Badge tone="neutral" className="text-[10px]">
                Coinstake
              </Badge>
            ) : (
              <Badge tone="neutral" className="text-[10px]">
                Transfer
              </Badge>
            )}
            {tx.blockHeight != null && (
              <span className="inline-flex items-center gap-0.5 text-[10px] text-fg-muted">
                <Layers className="h-3 w-3" aria-hidden />#{tx.blockHeight.toLocaleString()}
              </span>
            )}
          </div>
          <p className="mt-1 font-mono text-[11px] text-accent break-all">{shortTxid(tx.txid)}</p>
          {tx.time != null && (
            <p className="mt-1 text-[11px] text-fg-muted">{formatTransactionTime(tx.time)}</p>
          )}
        </div>
      </div>
    </ExplorerCardLink>
  );
}

export function BlockTransactionsSection({
  transactions,
  paging,
  offset,
  pageSize,
  onOffsetChange,
}: {
  transactions: IndexerTransactionSummary[];
  paging?: IndexerPaging | null;
  offset: number;
  pageSize: number;
  onOffsetChange: (next: number) => void;
}) {
  const [open, setOpen] = useState(true);
  const total = paging?.total ?? transactions.length;
  const page = Math.floor(offset / pageSize) + 1;
  const totalPages = Math.max(1, Math.ceil(total / pageSize));

  return (
    <section className="mobile-panel overflow-hidden rounded-2xl border border-border bg-bg-panel/60">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        className="flex w-full items-start gap-3 px-4 py-3.5 text-left"
        aria-expanded={open}
      >
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <h3 className="text-sm font-semibold text-fg">Transactions</h3>
            <Badge tone="neutral" className="text-[10px]">
              {total.toLocaleString()}
            </Badge>
          </div>
          <p className="mt-0.5 text-[11px] text-fg-muted">
            Page {page} of {totalPages} · ordered by position in block
          </p>
        </div>
        <ChevronDown
          className={cn(
            'mt-0.5 h-5 w-5 shrink-0 text-fg-muted transition-transform',
            open && 'rotate-180'
          )}
          aria-hidden
        />
      </button>

      {open && (
        <div className="border-t border-border/60 px-3 pb-3 pt-2">
          {transactions.length === 0 ? (
            <p className="py-4 text-center text-xs text-fg-muted">No transactions in this page.</p>
          ) : (
            <ul className="space-y-2">
              {transactions.map((tx, i) => (
                <li key={tx.txid}>
                  <BlockTxRow tx={tx} index={offset + i} />
                </li>
              ))}
            </ul>
          )}

          {paging && (paging.hasMore || offset > 0) && (
            <div className="mt-3 flex gap-2">
              <Button
                type="button"
                variant="secondary"
                className="h-9 flex-1 rounded-xl text-xs"
                disabled={offset === 0}
                onClick={() => onOffsetChange(Math.max(0, offset - pageSize))}
              >
                Previous
              </Button>
              <Button
                type="button"
                variant="secondary"
                className="h-9 flex-1 rounded-xl text-xs"
                disabled={!paging.hasMore}
                onClick={() => onOffsetChange(offset + pageSize)}
              >
                Next
              </Button>
            </div>
          )}
        </div>
      )}
    </section>
  );
}
