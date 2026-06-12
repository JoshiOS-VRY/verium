import { useEffect, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useParams } from 'react-router-dom';
import { BlockHero } from '@/components/mobile/explorer/BlockHero';
import { BlockStatsStrip } from '@/components/mobile/explorer/BlockStatsStrip';
import { BlockTransactionsSection } from '@/components/mobile/explorer/BlockTransactionsSection';
import { ExplorerDetailShell } from '@/components/mobile/explorer/ExplorerDetailShell';
import { shortTxid } from '@/components/mobile/explorer/tx-detail-utils';
import { useActiveCoin } from '@/lib/coin/context';
import { coinQueryKey } from '@/lib/coin/profile';
import { buildBlockExplorerUrl, effectiveBlockExplorerTemplate } from '@/lib/explorer-links';
import { fetchIndexerBlock } from '@/lib/indexer-api';
import { useUserPreferences } from '@/lib/user-preferences';

const PAGE_SIZE = 25;

export function ExplorerBlockPage() {
  const { id: rawId } = useParams();
  const blockId = rawId?.trim() ?? '';
  const coin = useActiveCoin();
  const { prefs } = useUserPreferences();
  const blockTemplate = effectiveBlockExplorerTemplate(coin, prefs.explorer_block_url_template);
  const [offset, setOffset] = useState(0);

  useEffect(() => {
    setOffset(0);
  }, [blockId]);

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

  const txs = data.transactions ?? [];
  const paging = data.paging;
  const subtitle = shortTxid(block.hash, 14, 10);

  return (
    <ExplorerDetailShell
      title={`Block #${block.height.toLocaleString()}`}
      subtitle={subtitle}
      externalUrl={externalUrl}
    >
      <BlockHero block={block} />

      <BlockStatsStrip block={block} />

      <BlockTransactionsSection
        transactions={txs}
        paging={paging}
        offset={offset}
        pageSize={PAGE_SIZE}
        onOffsetChange={setOffset}
      />

      {data.trusted && (
        <p className="px-2 text-center text-[10px] text-fg-subtle">Indexed · trusted source</p>
      )}
    </ExplorerDetailShell>
  );
}
