import { Blocks } from 'lucide-react';
import { useEffect, useMemo, useRef, useState } from 'react';

import { useQuery } from '@tanstack/react-query';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';

import { ExplorerLink } from '@/components/ExplorerLink';

import { BlockFoundBanner, MinedBlocksSummary } from '@/components/YouMinedCelebration';

import { StakeFoundBanner, StakedRewardsSummary } from '@/components/YouStakedCelebration';

import {
  buildRecentBlockRowModel,
  RecentBlockCard,
  RecentBlockTableRow,
} from '@/components/ExplorerRecentBlockRow';

import { useWalletMiningContext } from '@/hooks/useWalletMiningContext';
import { useWalletStakingContext } from '@/hooks/useWalletStakingContext';

import { subscribeBlockMined } from '@/hooks/useBlockMinedWatcher';
import { subscribeStakeReward } from '@/hooks/useStakeRewardWatcher';
import { useBlockRowEnterAnimation } from '@/hooks/useBlockRowEnterAnimation';
import { useChainSynced } from '@/hooks/useChainSynced';

import { useBlockAgeTick } from '@/hooks/useBlockAgeTick';
import { useEffectiveLocalChainTip } from '@/hooks/useEffectiveLocalChainTip';

import { fetchExplorerBlocks, EXPLORER_BLOCKS_POLL_MS, isExplorerApiEnabled } from '@/lib/explorer-api';
import type { ExplorerBlock } from '@/lib/explorer-api';
import {
  blockNeedsRpcEnrichment,
  blockRowFromRewardEvent,
  buildPendingBlocksAbove,
  enrichBlocksFromExplorer,
  enrichBlocksFromRpc,
  isPlaceholderBlockHash,
  MAX_PENDING_BLOCKS_ABOVE,
  mergeRecentBlocks,
} from '@/lib/local-recent-block';
import { useResponsiveLayout } from '@/hooks/useResponsiveLayout';

import { explorerBlocksHash } from '@/lib/explorer-links';
import { coinQueryKey } from '@/lib/coin/profile';
import { formatCoinAmount } from '@/lib/units';
import { cn } from '@/lib/utils';
import { useWindowVisible } from '@/hooks/useWindowVisible';

interface ExplorerRecentBlocksProps {
  coin: import('@/lib/coin/profile').CoinId;

  variant?: 'default' | 'dashboard';

  className?: string;
}

/**
 * Live rows come from the local node tip watcher (+ RPC gap backfill when the
 * wallet is ahead of the explorer index). The explorer query enriches rows with
 * miner address and reward once the indexer catches up (see explorer-v2
 * `useLatestBlocksPoll` + `liveBlocksMerge`).
 */
const BLOCKS_INDEXING_REFETCH_MS = 2_000;
const BLOCKS_ENRICH_RETRY_MS = 3_000;

const CELEBRATION_DISMISS_MS = 60_000;

function parseBlockOutput(value?: string): number {
  const n = Number(value);

  return Number.isFinite(n) ? n : 0;
}

interface CelebrationState {
  height: number;

  reward: string;
}

export function ExplorerRecentBlocks({
  coin,

  variant = 'default',

  className,
}: ExplorerRecentBlocksProps) {
  const isDashboard = variant === 'dashboard';

  const isVerium = coin === 'verium';

  const miningCtx = useWalletMiningContext(isVerium);
  const stakingCtx = useWalletStakingContext(!isVerium);

  const { synced } = useChainSynced(coin);

  const visible = useWindowVisible();
  const { isLight, isPhoneLayout } = useResponsiveLayout();

  const {
    height: localTipHeight,
    hash: localTipHash,
    time: localTipTime,
    chainTip,
  } = useEffectiveLocalChainTip(coin);
  const { enteringHash, nudgeOthers } = useBlockRowEnterAnimation(localTipHash);

  const ageTick = useBlockAgeTick(isDashboard && visible);

  const [celebration, setCelebration] = useState<CelebrationState | null>(null);

  const [localBlocks, setLocalBlocks] = useState<ExplorerBlock[]>([]);

  const dismissTimer = useRef<number | null>(null);

  useEffect(() => {
    setCelebration(null);
    setLocalBlocks([]);
  }, [coin]);

  const enabled = useQuery({
    queryKey: ['explorer-api-enabled'],

    queryFn: isExplorerApiEnabled,

    staleTime: Infinity,
  });

  const blocksPollMs = EXPLORER_BLOCKS_POLL_MS;

  const blocks = useQuery({
    queryKey: coinQueryKey(coin, 'explorer-blocks', 10),

    queryFn: () => fetchExplorerBlocks(coin, 10),

    enabled: (isDashboard || enabled.data === true) && visible,

    staleTime: blocksPollMs,

    refetchInterval: visible ? blocksPollMs : false,

    refetchOnWindowFocus: !isDashboard,

    retry: isDashboard ? 2 : 0,
  });

  const explorerTopHeight = blocks.data?.[0]?.height;
  const indexingLag =
    localTipHeight != null && explorerTopHeight != null && localTipHeight > explorerTopHeight;

  useEffect(() => {
    if (!visible || !indexingLag) return;

    void blocks.refetch();

    const fastPoll = window.setInterval(() => {
      void blocks.refetch();
    }, BLOCKS_INDEXING_REFETCH_MS);

    return () => window.clearInterval(fastPoll);
  }, [blocks.refetch, indexingLag, visible]);

  const explorerBlocksSignature = blocks.data?.map((block) => block.height).join(',') ?? '';

  useEffect(() => {
    if (!visible) return;

    const explorerTop = blocks.data?.[0]?.height ?? 0;
    const heights = new Set<number>();

    if (localTipHeight != null && localTipHeight > explorerTop) {
      const gap = localTipHeight - explorerTop;
      const cap = Math.min(gap, MAX_PENDING_BLOCKS_ABOVE);
      for (let i = 0; i < cap; i += 1) {
        heights.add(localTipHeight - i);
      }
    }

    for (const block of blocks.data ?? []) {
      if (blockNeedsRpcEnrichment(block, coin)) {
        heights.add(block.height);
      }
    }

    if (heights.size === 0) return;

    let cancelled = false;

    const targetForHeight = (height: number): ExplorerBlock => {
      const fromExplorer = blocks.data?.find((block) => block.height === height);
      if (fromExplorer) return fromExplorer;
      return {
        id: height,
        hash:
          height === localTipHeight && localTipHash
            ? localTipHash
            : isLight
              ? `light-pending-${height}`
              : `local-pending-${height}`,
        height,
        time: height === localTipHeight ? localTipTime : 0,
      };
    };

    const targets = [...heights].map(targetForHeight);

    const enrich = isLight ? enrichBlocksFromExplorer : enrichBlocksFromRpc;
    const runEnrich = () => {
      void enrich(coin, targets).then((rows) => {
        if (cancelled || rows.length === 0) return;
        setLocalBlocks((prev) => mergeRecentBlocks(rows, prev, 24));
      });
    };

    runEnrich();
    const retry = window.setInterval(runEnrich, BLOCKS_ENRICH_RETRY_MS);

    return () => {
      cancelled = true;
      window.clearInterval(retry);
    };
  }, [
    blocks.data,
    coin,
    explorerBlocksSignature,
    isLight,
    localTipHash,
    localTipHeight,
    localTipTime,
    visible,
  ]);

  useEffect(() => {
    const indexed = blocks.data ?? [];
    if (indexed.length === 0) return;
    setLocalBlocks((prev) => {
      const next = prev.filter((local) => {
        const explorerRow = indexed.find((row) => row.height === local.height);
        if (!explorerRow) return true;
        return !(isPlaceholderBlockHash(local.hash) && !isPlaceholderBlockHash(explorerRow.hash));
      });
      return next.length === prev.length ? prev : next;
    });
  }, [blocks.data]);

  useEffect(() => {
    if (coin === 'verium') {
      return subscribeBlockMined((event) => {
        setCelebration({
          height: event.height,
          reward:
            event.amount != null && Number.isFinite(event.amount)
              ? formatCoinAmount(event.amount, 'verium', 4)
              : '—',
        });

        void blockRowFromRewardEvent('verium', event).then((row) => {
          if (!row) return;
          setLocalBlocks((prev) => mergeRecentBlocks([row], prev, 12));
        });
      });
    }

    return subscribeStakeReward((event) => {
      setCelebration({
        height: event.height,
        reward:
          event.amount != null && Number.isFinite(event.amount)
            ? formatCoinAmount(event.amount, 'vericoin', 4)
            : '—',
      });

      void blockRowFromRewardEvent('vericoin', event).then((row) => {
        if (!row) return;
        setLocalBlocks((prev) => mergeRecentBlocks([row], prev, 12));
      });
    });
  }, [coin]);

  useEffect(() => {
    if (!synced) setCelebration(null);
  }, [synced]);

  useEffect(() => {
    if (!celebration) return;

    if (dismissTimer.current != null) {
      window.clearTimeout(dismissTimer.current);
    }

    dismissTimer.current = window.setTimeout(() => {
      setCelebration(null);

      dismissTimer.current = null;
    }, CELEBRATION_DISMISS_MS);

    return () => {
      if (dismissTimer.current != null) {
        window.clearTimeout(dismissTimer.current);

        dismissTimer.current = null;
      }
    };
  }, [celebration]);

  const loading = enabled.isLoading || blocks.isLoading;

  const feedLimit = 10;
  const explorerBlocks = blocks.data ?? [];
  const liveAndLocal = useMemo(() => {
    return mergeRecentBlocks(chainTip.recentBlocks, localBlocks, 24);
  }, [chainTip.recentBlocks, localBlocks]);

  const pendingLocal = useMemo(() => {
    const explorerTop = explorerBlocks[0]?.height ?? 0;
    if (localTipHeight == null || localTipHeight <= explorerTop) {
      return [];
    }

    const knownHeights = new Set<number>();
    for (const block of explorerBlocks) knownHeights.add(block.height);
    for (const block of liveAndLocal) knownHeights.add(block.height);

    return buildPendingBlocksAbove(
      explorerTop,
      localTipHeight,
      localTipHash,
      localTipTime,
      knownHeights
    );
  }, [explorerBlocks, liveAndLocal, localTipHash, localTipHeight, localTipTime]);

  const blockRows = useMemo(() => {
    const merged = mergeRecentBlocks(explorerBlocks, pendingLocal, feedLimit + 8);
    return mergeRecentBlocks(merged, liveAndLocal, feedLimit);
  }, [explorerBlocks, feedLimit, liveAndLocal, pendingLocal]);

  /** Same tip source as the dashboard hero. */
  const tipHeight = localTipHeight ?? blockRows[0]?.height;

  if (!isDashboard && enabled.data !== true) return null;

  const rowContext = {
    coin,
    tipHeight,
    enteringHash,
    nudgeOthers,
    miningCtx,
    stakingCtx,
  };

  const yoursInFeed = blockRows.filter(
    (block) => buildRecentBlockRowModel(block, rowContext).isYours
  );

  const yoursRewardTotal = yoursInFeed.reduce(
    (sum, block) => sum + parseBlockOutput(block.output_total ?? block.mint),
    0
  );

  return (
    <Card
      className={cn(
        isDashboard && 'flex  flex-col',

        className
      )}
    >
      <CardHeader className="flex-row items-start justify-between shrink-0">
        <div className="space-y-2">
          <CardTitle className="flex items-center gap-2 text-sm font-semibold uppercase tracking-wide text-fg">
            <Blocks className="h-4 w-4 shrink-0 text-accent" aria-hidden />
            Recent blocks
          </CardTitle>

          <CardDescription className="flex flex-wrap items-center gap-x-2 gap-y-1.5">
            {!loading && yoursInFeed.length > 0 && isVerium && (
              <MinedBlocksSummary count={yoursInFeed.length} totalRewardVrm={yoursRewardTotal} />
            )}
            {!loading && yoursInFeed.length > 0 && !isVerium && (
              <StakedRewardsSummary count={yoursInFeed.length} />
            )}
          </CardDescription>
        </div>

        <ExplorerLink
          coin={coin}
          target={{ kind: 'raw', url: explorerBlocksHash(coin) }}
          label="All blocks"
        />
      </CardHeader>

      <CardContent className={cn('p-0', isDashboard && 'flex min-h-0 flex-1 flex-col')}>
        {celebration && synced && isVerium && (
          <div className="shrink-0 pt-1">
            <BlockFoundBanner
              height={celebration.height}
              reward={celebration.reward}
              onDismiss={() => setCelebration(null)}
            />
          </div>
        )}
        {celebration && synced && !isVerium && (
          <div className="shrink-0 pt-1">
            <StakeFoundBanner
              height={celebration.height}
              reward={celebration.reward}
              onDismiss={() => setCelebration(null)}
            />
          </div>
        )}

        {blocks.isError ? (
          <div className="px-4 py-6 text-xs text-fg-subtle">
            Could not load blocks from explorer.
            {blocks.error != null && <div className="mt-1 text-danger">{String(blocks.error)}</div>}
          </div>
        ) : isPhoneLayout ? (
          <div
            className={cn(
              'relative isolate overflow-x-hidden overflow-y-auto px-3 pb-3 mobile-recent-blocks-scroll',
              isDashboard ? 'flex-1' : 'max-h-[480px]'
            )}
          >
            <div
              className={cn(
                'flex flex-col gap-2.5',
                isDashboard && 'mobile-recent-blocks-grid'
              )}
            >
              {loading &&
                Array.from({ length: isDashboard ? 8 : 5 }).map((_, i) => (
                  <div
                    key={`loading-card-${i}`}
                    className="h-28 animate-pulse rounded-xl border border-border bg-bg-subtle"
                  />
                ))}

              {!loading &&
                blockRows.map((block) => (
                  <RecentBlockCard
                    key={`block-${block.height}`}
                    coin={coin}
                    model={buildRecentBlockRowModel(block, rowContext)}
                    isDashboard={isDashboard}
                    ageTick={ageTick}
                  />
                ))}

              {!loading && blockRows.length === 0 && (
                <p className="py-8 text-center text-sm text-fg-subtle">No blocks returned.</p>
              )}
            </div>
          </div>
        ) : (
          <div
            className={cn(
              'relative isolate overflow-auto',
              isDashboard ? 'flex-1' : 'max-h-[360px]'
            )}
          >
            <table className="w-full border-collapse text-sm">
              <thead className="sticky top-0 z-10 bg-bg-panel text-xs uppercase text-fg-subtle">
                <tr>
                  <th className="px-4 py-2 text-left font-medium">Height</th>
                  <th className="px-4 py-2 text-right font-medium">Time</th>
                  <th className="px-4 py-2 text-right font-medium">Txs</th>
                  <th className="px-4 py-2 text-right font-medium">Out</th>
                  {isDashboard && (
                    <>
                      <th className="hidden px-4 py-2 text-right font-medium sm:table-cell">
                        Size
                      </th>
                      <th className="hidden px-4 py-2 text-right font-medium md:table-cell">
                        Difficulty
                      </th>
                    </>
                  )}
                  <th className="px-4 py-2 text-left font-medium">Extracted by</th>
                </tr>
              </thead>

              <tbody>
                {loading &&
                  Array.from({ length: isDashboard ? 12 : 6 }).map((_, i) => (
                    <tr key={`loading-${i}`} className="border-t border-border">
                      <td colSpan={isDashboard ? 7 : 5} className="px-4 py-2">
                        <div className="h-4 animate-pulse rounded bg-bg-subtle" />
                      </td>
                    </tr>
                  ))}

                {!loading &&
                  blockRows.map((block) => (
                    <RecentBlockTableRow
                      key={`block-${block.height}`}
                      coin={coin}
                      model={buildRecentBlockRowModel(block, rowContext)}
                      isDashboard={isDashboard}
                      ageTick={ageTick}
                    />
                  ))}

                {!loading && blockRows.length === 0 && (
                  <tr>
                    <td
                      colSpan={isDashboard ? 7 : 5}
                      className="px-4 py-6 text-center text-sm text-fg-subtle"
                    >
                      No blocks returned.
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
