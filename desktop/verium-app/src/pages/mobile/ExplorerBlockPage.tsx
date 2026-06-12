import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useParams } from 'react-router-dom';
import { Button } from '@/components/ui/Button';
import { ExplorerDetailShell } from '@/components/mobile/explorer/ExplorerDetailShell';
import { ExplorerInternalLink } from '@/components/mobile/explorer/ExplorerInternalLink';
import { useActiveCoin } from '@/lib/coin/context';
import { coinQueryKey } from '@/lib/coin/profile';
import { buildBlockExplorerUrl, effectiveBlockExplorerTemplate } from '@/lib/explorer-links';
import { fetchIndexerBlock } from '@/lib/indexer-api';
import { explorerBlockPath, explorerTxPath } from '@/lib/explorer-nav';
import { useUserPreferences } from '@/lib/user-preferences';

const PAGE_SIZE = 25;

export function ExplorerBlockPage() {
  const { id: rawId } = useParams();
  const blockId = rawId?.trim() ?? '';
  const coin = useActiveCoin();
  const { prefs } = useUserPreferences();
  const blockTemplate = effectiveBlockExplorerTemplate(coin, prefs.explorer_block_url_template);
  const [offset, setOffset] = useState(0);

  const detail = useQuery({
    queryKey: coinQueryKey(coin, 'indexer-block', blockId, offset),
    queryFn: () => fetchIndexerBlock(coin, blockId, PAGE_SIZE, offset),
    enabled: blockId.length > 0,
  });

  if (!blockId) {
    return (
      <ExplorerDetailShell title="Block" subtitle="Missing block id.">
        <p className="px-3 text-sm text-fg-muted">Go back and pick a block from the dashboard.</p>
      </ExplorerDetailShell>
    );
  }

  if (detail.isLoading) {
    return (
      <ExplorerDetailShell title="Block" subtitle="Loading from explorer index…">
        <p className="px-3 text-sm text-fg-muted">Fetching block details…</p>
      </ExplorerDetailShell>
    );
  }

  if (detail.isError || !detail.data) {
    return (
      <ExplorerDetailShell
        title="Block"
        subtitle="Could not load this block."
        externalUrl={buildBlockExplorerUrl(coin, blockTemplate, blockId)}
      >
        <p className="px-3 text-sm text-danger">
          {detail.error instanceof Error ? detail.error.message : 'Indexer request failed.'}
        </p>
      </ExplorerDetailShell>
    );
  }

  const data = detail.data;
  const block = data.block;
  const externalUrl = buildBlockExplorerUrl(coin, blockTemplate, blockId);

  if (!data.found || !block) {
    return (
      <ExplorerDetailShell
        title="Block not indexed"
        subtitle="This block is not in the explorer index yet."
        externalUrl={externalUrl}
      >
        <p className="px-3 font-mono text-xs break-all text-fg-muted">{blockId}</p>
      </ExplorerDetailShell>
    );
  }

  const when = block.time ? new Date(block.time * 1000).toLocaleString() : '—';
  const txs = data.transactions ?? [];
  const paging = data.paging;

  return (
    <ExplorerDetailShell
      title={`Block #${block.height}`}
      subtitle={block.hash.slice(0, 18) + '…'}
      externalUrl={externalUrl}
    >
      <section className="mobile-panel space-y-2 rounded-2xl border border-border bg-bg-panel/60 p-4">
        <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-2 text-xs">
          <dt className="text-fg-muted">Time</dt>
          <dd>{when}</dd>
          <dt className="text-fg-muted">Transactions</dt>
          <dd>{block.txCount ?? txs.length}</dd>
          {block.size != null && (
            <>
              <dt className="text-fg-muted">Size</dt>
              <dd>{block.size.toLocaleString()} bytes</dd>
            </>
          )}
          {block.difficulty && (
            <>
              <dt className="text-fg-muted">Difficulty</dt>
              <dd className="font-mono text-[11px] break-all">{block.difficulty}</dd>
            </>
          )}
          <dt className="text-fg-muted">Hash</dt>
          <dd className="font-mono text-[11px] break-all">{block.hash}</dd>
        </dl>
        <div className="flex flex-wrap gap-2 pt-1">
          {block.previousHash && (
            <ExplorerInternalLink to={explorerBlockPath(block.previousHash)} className="text-xs">
              Previous block
            </ExplorerInternalLink>
          )}
          {block.nextHash && (
            <ExplorerInternalLink to={explorerBlockPath(block.nextHash)} className="text-xs">
              Next block
            </ExplorerInternalLink>
          )}
        </div>
      </section>

      <section className="mobile-panel rounded-2xl border border-border bg-bg-panel/60 p-4">
        <h3 className="text-sm font-medium text-fg">
          Transactions
          {paging ? ` (${paging.total})` : ''}
        </h3>
        {txs.length === 0 ? (
          <p className="mt-2 text-xs text-fg-muted">No transactions in this page.</p>
        ) : (
          <ul className="mt-2 space-y-2">
            {txs.map((tx) => (
              <li key={tx.txid} className="rounded-lg border border-border/60 px-3 py-2 text-xs">
                <ExplorerInternalLink to={explorerTxPath(tx.txid)} mono className="block">
                  {tx.txid}
                </ExplorerInternalLink>
                {tx.time && (
                  <p className="mt-1 text-fg-muted">{new Date(tx.time * 1000).toLocaleString()}</p>
                )}
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
              onClick={() => setOffset((o) => Math.max(0, o - PAGE_SIZE))}
            >
              Previous
            </Button>
            <Button
              type="button"
              variant="secondary"
              className="h-9 flex-1 rounded-xl text-xs"
              disabled={!paging.hasMore}
              onClick={() => setOffset((o) => o + PAGE_SIZE)}
            >
              Next
            </Button>
          </div>
        )}
      </section>
    </ExplorerDetailShell>
  );
}
