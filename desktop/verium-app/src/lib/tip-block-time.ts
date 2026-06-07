import type { ExplorerBlock } from "@/lib/explorer-api";
import type { ChainTip } from "@/lib/chain-tip-store";

/** Resolve the tip block's unix timestamp — never use chain `mediantime` for "mined ago". */
export function resolveTipBlockTime(
  tipHeight: number | undefined,
  sources: {
    chainTip?: ChainTip | null;
    explorerBlocks?: ExplorerBlock[];
    headerTime?: number | null;
  },
): number | undefined {
  if (tipHeight == null || tipHeight <= 0) return undefined;

  const fromWatcher =
    sources.chainTip?.height === tipHeight && sources.chainTip.time > 0
      ? sources.chainTip.time
      : undefined;
  if (fromWatcher != null) return fromWatcher;

  const rows = sources.explorerBlocks;
  if (rows?.length) {
    const exact = rows.find((b) => b.height === tipHeight);
    if (exact?.time != null && exact.time > 0) return exact.time;
    const newest = rows[0];
    if (newest?.height === tipHeight && newest.time > 0) return newest.time;
  }

  if (sources.headerTime != null && sources.headerTime > 0) {
    return sources.headerTime;
  }

  return undefined;
}
