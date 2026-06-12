import { useMemo, useState, type ReactNode } from 'react';
import { ChevronDown, Hammer, Link2, Wallet } from 'lucide-react';
import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import type { IndexerVin, IndexerVout } from '@/lib/indexer-api';
import { formatIndexerAmount } from '@/lib/indexer-api';
import {
  formatIndexerCoinsTotal,
  indexerTicker,
  sumIndexerAmountCoins,
} from '@/lib/indexer-amount';
import { explorerAddressPath, explorerTxPath } from '@/lib/explorer-nav';
import { cn } from '@/lib/utils';
import { ExplorerInternalLink } from './ExplorerInternalLink';
import { shortExplorerAddress, shortTxid } from './tx-detail-utils';

const DEFAULT_VISIBLE = 8;
const EXPAND_STEP = 20;

function IoSectionShell({
  title,
  description,
  totalLabel,
  total,
  ticker,
  count,
  defaultOpen,
  tone,
  children,
}: {
  title: string;
  description?: string;
  totalLabel: string;
  total: number;
  ticker: string;
  count: number;
  defaultOpen?: boolean;
  tone: 'input' | 'output';
  children: ReactNode;
}) {
  const [open, setOpen] = useState(defaultOpen ?? count <= DEFAULT_VISIBLE);

  return (
    <section
      className={cn(
        'mobile-panel overflow-hidden rounded-2xl border bg-bg-panel/60',
        tone === 'input' ? 'border-amber-500/20' : 'border-emerald-500/20'
      )}
    >
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        className="flex w-full items-start gap-3 px-4 py-3.5 text-left"
        aria-expanded={open}
      >
        <div
          className={cn(
            'mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-lg',
            tone === 'input' ? 'bg-amber-500/15 text-amber-700 dark:text-amber-300' : 'bg-emerald-500/15 text-emerald-700 dark:text-emerald-300'
          )}
        >
          {tone === 'input' ? (
            <Link2 className="h-4 w-4" aria-hidden />
          ) : (
            <Wallet className="h-4 w-4" aria-hidden />
          )}
        </div>
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <h3 className="text-sm font-semibold text-fg">{title}</h3>
            <Badge tone="neutral" className="text-[10px]">
              {count}
            </Badge>
          </div>
          {description ? (
            <p className="mt-0.5 text-[11px] leading-relaxed text-fg-muted">{description}</p>
          ) : null}
          <p className="mt-1.5 text-xs text-fg-muted">
            {totalLabel}{' '}
            <span className="font-semibold tabular-nums text-fg">
              {formatIndexerCoinsTotal(total, ticker)}
            </span>
          </p>
        </div>
        <ChevronDown
          className={cn(
            'mt-1 h-5 w-5 shrink-0 text-fg-muted transition-transform',
            open && 'rotate-180'
          )}
          aria-hidden
        />
      </button>
      {open ? <div className="border-t border-border/60 px-3 pb-3 pt-2">{children}</div> : null}
    </section>
  );
}

function ExpandableList<T>({
  items,
  renderItem,
  getKey,
}: {
  items: T[];
  renderItem: (item: T) => ReactNode;
  getKey: (item: T) => string | number;
}) {
  const [visible, setVisible] = useState(DEFAULT_VISIBLE);
  const shown = items.slice(0, visible);
  const remaining = items.length - shown.length;

  return (
    <div className="space-y-2">
      {shown.map((item) => (
        <div key={getKey(item)}>{renderItem(item)}</div>
      ))}
      {remaining > 0 && (
        <Button
          type="button"
          variant="secondary"
          size="sm"
          className="h-9 w-full rounded-xl text-xs"
          onClick={() => setVisible((n) => Math.min(items.length, n + EXPAND_STEP))}
        >
          Show {Math.min(remaining, EXPAND_STEP)} more ({remaining} hidden)
        </Button>
      )}
      {visible > DEFAULT_VISIBLE && remaining === 0 && items.length > DEFAULT_VISIBLE && (
        <button
          type="button"
          className="w-full text-center text-[11px] text-accent underline-offset-2 hover:underline"
          onClick={() => setVisible(DEFAULT_VISIBLE)}
        >
          Collapse list
        </button>
      )}
    </div>
  );
}

function VinRow({ vin }: { vin: IndexerVin }) {
  const isCoinbase = !vin.prevTxid && !vin.address;
  const label = isCoinbase ? 'Coinbase' : vin.address ? 'From address' : 'Previous output';

  return (
    <article className="rounded-xl border border-border/70 bg-bg-subtle/30 px-3 py-2.5">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-1.5">
            <span className="text-[10px] font-semibold uppercase tracking-wide text-fg-muted">
              Input {vin.n}
            </span>
            {isCoinbase && (
              <Badge tone="warning" className="gap-0.5 text-[10px]">
                <Hammer className="h-3 w-3" aria-hidden />
                Mint
              </Badge>
            )}
            {!vin.resolved && !isCoinbase && (
              <Badge tone="neutral" className="text-[10px]">Unresolved</Badge>
            )}
          </div>
          <p className="mt-0.5 text-[11px] text-fg-subtle">{label}</p>
        </div>
        <p className="shrink-0 text-sm font-semibold tabular-nums text-fg">
          {formatIndexerAmount(vin.value ?? undefined)}
        </p>
      </div>
      <div className="mt-2 border-t border-border/50 pt-2">
        {vin.address ? (
          <ExplorerInternalLink to={explorerAddressPath(vin.address)} mono className="block">
            {shortExplorerAddress(vin.address)}
          </ExplorerInternalLink>
        ) : vin.prevTxid ? (
          <div className="space-y-0.5">
            <ExplorerInternalLink to={explorerTxPath(vin.prevTxid)} mono className="block">
              {shortTxid(vin.prevTxid)}
            </ExplorerInternalLink>
            <p className="text-[10px] text-fg-muted">Spends output #{vin.prevVout ?? '?'}</p>
          </div>
        ) : (
          <p className="text-[11px] text-fg-muted">No source address (newly minted coins)</p>
        )}
      </div>
    </article>
  );
}

function VoutRow({ vout }: { vout: IndexerVout }) {
  return (
    <article className="rounded-xl border border-border/70 bg-bg-subtle/30 px-3 py-2.5">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-1.5">
            <span className="text-[10px] font-semibold uppercase tracking-wide text-fg-muted">
              Output {vout.n}
            </span>
            {vout.isSpent && (
              <Badge tone="neutral" className="text-[10px]">Spent later</Badge>
            )}
          </div>
          <p className="mt-0.5 text-[11px] text-fg-subtle">To address</p>
        </div>
        <p className="shrink-0 text-sm font-semibold tabular-nums text-fg">
          {formatIndexerAmount(vout.value ?? undefined)}
        </p>
      </div>
      <div className="mt-2 border-t border-border/50 pt-2">
        {vout.address ? (
          <ExplorerInternalLink to={explorerAddressPath(vout.address)} mono className="block">
            {shortExplorerAddress(vout.address)}
          </ExplorerInternalLink>
        ) : (
          <p className="text-[11px] text-fg-muted">No decoded address</p>
        )}
      </div>
    </article>
  );
}

export function TransactionInputsSection({ inputs }: { inputs: IndexerVin[] }) {
  const total = sumIndexerAmountCoins(inputs.map((v) => v.value));
  const ticker = indexerTicker(inputs.find((v) => v.value)?.value) || 'VRM';
  const sorted = useMemo(() => [...inputs].sort((a, b) => a.n - b.n), [inputs]);

  if (sorted.length === 0) return null;

  return (
    <IoSectionShell
      title="Inputs"
      description="Coins spent from previous outputs or minted (coinbase)."
      totalLabel="Total in"
      total={total}
      ticker={ticker}
      count={sorted.length}
      tone="input"
      defaultOpen={sorted.length <= 4}
    >
      <ExpandableList
        items={sorted}
        getKey={(vin) => vin.n}
        renderItem={(vin) => <VinRow vin={vin} />}
      />
    </IoSectionShell>
  );
}

export function TransactionOutputsSection({ outputs }: { outputs: IndexerVout[] }) {
  const total = sumIndexerAmountCoins(outputs.map((v) => v.value));
  const ticker = indexerTicker(outputs.find((v) => v.value)?.value) || 'VRM';
  const sorted = useMemo(() => [...outputs].sort((a, b) => a.n - b.n), [outputs]);

  if (sorted.length === 0) return null;

  return (
    <IoSectionShell
      title="Outputs"
      description="New outputs created — payment, change, or stake reward destinations."
      totalLabel="Total out"
      total={total}
      ticker={ticker}
      count={sorted.length}
      tone="output"
      defaultOpen={sorted.length <= 4}
    >
      <ExpandableList
        items={sorted}
        getKey={(vout) => vout.n}
        renderItem={(vout) => <VoutRow vout={vout} />}
      />
    </IoSectionShell>
  );
}
