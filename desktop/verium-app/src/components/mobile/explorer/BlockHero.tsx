import { Check, ChevronLeft, ChevronRight, Copy, Layers } from 'lucide-react';
import { AnimatedBlockNumber } from '@/components/AnimatedBlockNumber';
import { Button } from '@/components/ui/Button';
import { useCopyToClipboard } from '@/hooks/useCopyToClipboard';
import { useBlockAgeTick } from '@/hooks/useBlockAgeTick';
import { explorerBlockPath } from '@/lib/explorer-nav';
import type { IndexerBlockSummary } from '@/lib/indexer-api';
import { formatTransactionTime } from '@/lib/units';
import { cn, formatBlockAge } from '@/lib/utils';
import { ExplorerInternalLink } from './ExplorerInternalLink';
import { shortTxid } from './tx-detail-utils';

export function BlockHero({ block }: { block: IndexerBlockSummary }) {
  const { copied, copy } = useCopyToClipboard();
  const tick = useBlockAgeTick(block.time != null);
  const age = block.time != null ? formatBlockAge(block.time, tick) : null;
  const when = block.time != null ? formatTransactionTime(block.time) : '—';

  return (
    <section className="mobile-panel overflow-hidden rounded-2xl border border-border bg-gradient-to-br from-bg-panel via-bg-panel to-accent/5 p-4 shadow-sm">
      <div className="flex items-start gap-3">
        <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl border border-border bg-bg-subtle/50">
          <Layers className="h-5 w-5 text-accent" aria-hidden />
        </div>
        <div className="min-w-0 flex-1">
          <p className="text-[11px] font-medium uppercase tracking-wide text-fg-subtle">
            Block height
          </p>
          <AnimatedBlockNumber
            value={block.height}
            className="mt-0.5 text-3xl font-bold tabular-nums tracking-tight text-fg"
            animateOnIncrease={false}
          />
          <p className="mt-1 text-xs text-fg-muted">{when}</p>
          {age && (
            <p className="mt-0.5 text-[11px] text-fg-subtle">
              Mined <span className="font-medium text-fg-muted">{age}</span> ago
            </p>
          )}
        </div>
      </div>

      <div className="mt-4 flex items-center gap-2 rounded-xl border border-border/70 bg-bg-subtle/40 px-3 py-2">
        <p className="min-w-0 flex-1 font-mono text-[11px] leading-relaxed break-all text-fg-muted">
          {shortTxid(block.hash, 16, 12)}
        </p>
        <Button
          type="button"
          variant="secondary"
          size="sm"
          className={cn(
            'h-8 shrink-0 rounded-lg px-2.5',
            copied && 'border-success/40 text-success'
          )}
          onClick={() => void copy(block.hash)}
        >
          {copied ? <Check className="h-3.5 w-3.5" /> : <Copy className="h-3.5 w-3.5" />}
          <span className="text-[11px]">{copied ? 'Copied' : 'Copy hash'}</span>
        </Button>
      </div>

      {(block.previousHash || block.nextHash) && (
        <div className="mt-3 grid grid-cols-2 gap-2">
          {block.previousHash ? (
            <ExplorerInternalLink
              to={explorerBlockPath(block.previousHash)}
              className="flex items-center justify-center gap-1.5 rounded-xl border border-border/70 bg-bg-subtle/30 px-3 py-2.5 text-xs font-medium text-fg transition-colors hover:bg-bg-subtle/60"
            >
              <ChevronLeft className="h-4 w-4 shrink-0 text-fg-muted" aria-hidden />
              Previous
            </ExplorerInternalLink>
          ) : (
            <div className="rounded-xl border border-dashed border-border/50 px-3 py-2.5 text-center text-xs text-fg-subtle">
              No previous
            </div>
          )}
          {block.nextHash ? (
            <ExplorerInternalLink
              to={explorerBlockPath(block.nextHash)}
              className="flex items-center justify-center gap-1.5 rounded-xl border border-border/70 bg-bg-subtle/30 px-3 py-2.5 text-xs font-medium text-fg transition-colors hover:bg-bg-subtle/60"
            >
              Next
              <ChevronRight className="h-4 w-4 shrink-0 text-fg-muted" aria-hidden />
            </ExplorerInternalLink>
          ) : (
            <div className="rounded-xl border border-dashed border-border/50 px-3 py-2.5 text-center text-xs text-fg-subtle">
              Chain tip
            </div>
          )}
        </div>
      )}
    </section>
  );
}
