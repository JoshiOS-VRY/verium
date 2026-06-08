import type { ExplorerBlock } from "@/lib/explorer-api";
import type { ChainTip } from "@/lib/chain-tip-store";

/** Tip rows from Electrum height or pre-index stubs — not authoritative block times. */
export function isPlaceholderTipHash(hash: string | undefined): boolean {
  if (!hash) return true;
  return (
    hash.startsWith("light-tip-") ||
    hash.startsWith("light-pending-") ||
    hash.startsWith("local-pending-")
  );
}

function explorerTimeAtHeight(
  tipHeight: number,
  rows: ExplorerBlock[] | undefined,
): number | undefined {
  if (!rows?.length) return undefined;
  const exact = rows.find((block) => block.height === tipHeight);
  if (exact?.time != null && exact.time > 0) return exact.time;
  const newest = rows[0];
  if (newest?.height === tipHeight && newest.time > 0) return newest.time;
  return undefined;
}

/** Resolve the tip block's unix timestamp — never use chain `mediantime` for "mined ago". */
export function resolveTipBlockTime(
  tipHeight: number | undefined,
  sources: {
    chainTip?: Pick<ChainTip, "height" | "hash" | "time"> | null;
    explorerBlocks?: ExplorerBlock[];
    headerTime?: number | null;
  },
): number | undefined {
  if (tipHeight == null || tipHeight <= 0) return undefined;

  const fromExplorer = explorerTimeAtHeight(tipHeight, sources.explorerBlocks);
  if (fromExplorer != null) return fromExplorer;

  const watcher = sources.chainTip;
  if (
    watcher?.height === tipHeight &&
    watcher.time > 0 &&
    !isPlaceholderTipHash(watcher.hash)
  ) {
    return watcher.time;
  }

  if (sources.headerTime != null && sources.headerTime > 0) {
    return sources.headerTime;
  }

  return undefined;
}
