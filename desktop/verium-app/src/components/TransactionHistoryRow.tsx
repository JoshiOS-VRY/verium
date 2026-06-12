import { Badge } from '@/components/ui/Badge';
import { ConfirmationProgress } from '@/components/ConfirmationProgress';
import { ExplorerLink } from '@/components/ExplorerLink';
import type { CoinId } from '@/lib/coin/profile';
import type { TransactionItem } from '@/lib/rpc/client';
import {
  transactionCategoryBadgeClass,
  transactionCategoryLabel,
} from '@/lib/transaction-category';
import { formatCoinAmount, formatTransactionTime } from '@/lib/units';
import { cn } from '@/lib/utils';

export function TransactionHistoryCard({
  tx,
  coin,
  isPoolPayout,
}: {
  tx: TransactionItem;
  coin: CoinId;
  isPoolPayout?: boolean;
}) {
  return (
    <article
      className={cn(
        'min-w-0 max-w-full rounded-xl border border-border bg-bg-panel/60 px-3 py-3',
        'odd:bg-bg-subtle/30'
      )}
    >
      <div className="flex min-w-0 items-start justify-between gap-3">
        <div className="flex min-w-0 flex-wrap items-center gap-1.5">
          <Badge className={transactionCategoryBadgeClass(tx.category)}>
            {transactionCategoryLabel(tx.category)}
          </Badge>
          {isPoolPayout ? <Badge tone="neutral">Pool payout</Badge> : null}
        </div>
        <span
          className={cn(
            'shrink-0 text-right text-sm font-semibold tabular-nums',
            tx.amount > 0 ? 'text-success' : tx.amount < 0 ? 'text-fg' : 'text-fg-muted'
          )}
        >
          {formatCoinAmount(tx.amount, coin, 8)}
        </span>
      </div>

      <p className="mt-2 text-xs text-fg-muted">{formatTransactionTime(tx.time)}</p>

      {tx.address ? (
        <p className="mt-1.5 break-all font-mono text-xs text-fg-muted">{tx.address}</p>
      ) : null}

      <div className="mt-2.5 flex min-w-0 items-center justify-between gap-2 border-t border-border/60 pt-2">
        <ConfirmationProgress confirmations={tx.confirmations} category={tx.category} />
        <ExplorerLink
          coin={coin}
          target={{ kind: 'tx', txid: tx.txid }}
          label="View"
          title={`Open tx ${tx.txid} on the explorer`}
          className="shrink-0 text-[11px]"
        />
      </div>
    </article>
  );
}
