import type { CoinId } from "@/lib/coin/profile";
import type { ExplorerBlock } from "@/lib/explorer-api";
import type { BlockMinedEvent } from "@/hooks/useBlockMinedWatcher";
import type { StakeRewardEvent } from "@/hooks/useStakeRewardWatcher";
import {
  fetchExplorerBlocksForFeed,
  fetchLocalBlocksForFeed,
} from "@/lib/explorer-api";
import { isPlaceholderTipHash } from "@/lib/tip-block-time";
import { rpcRaw } from "@/lib/rpc/client";

export type WalletRewardEvent = BlockMinedEvent | StakeRewardEvent;

/** Merge explorer feed with blocks learned from the local node (shown immediately after mining). */
export function mergeRecentBlocks(
  explorer: ExplorerBlock[],
  local: ExplorerBlock[],
  limit = 10,
): ExplorerBlock[] {
  const byHeight = new Map<number, ExplorerBlock>();
  for (const block of explorer) {
    byHeight.set(block.height, block);
  }
  for (const block of local) {
    const existing = byHeight.get(block.height);
    if (!existing) {
      byHeight.set(block.height, block);
      continue;
    }
    byHeight.set(block.height, {
      ...existing,
      ...block,
      miner_address: block.miner_address ?? existing.miner_address,
      output_total: block.output_total ?? existing.output_total,
      mint: block.mint ?? existing.mint,
      output_count: block.output_count ?? existing.output_count,
      size: block.size ?? existing.size,
      difficulty: block.difficulty ?? existing.difficulty,
      n_tx: block.n_tx ?? existing.n_tx,
      time: block.time || existing.time,
    });
  }
  return [...byHeight.values()]
    .sort((a, b) => b.height - a.height)
    .slice(0, limit);
}

function voutAddress(vout: unknown): string | undefined {
  if (!vout || typeof vout !== "object") return undefined;
  const vo = vout as Record<string, unknown>;
  const spk = vo.scriptPubKey as Record<string, unknown> | undefined;
  if (typeof spk?.address === "string") return spk.address;
  const addresses = spk?.addresses;
  if (Array.isArray(addresses) && typeof addresses[0] === "string") {
    return addresses[0];
  }
  return undefined;
}

function sumBlockOutputs(block: Record<string, unknown>): {
  total: number;
  count: number;
} {
  const txs = Array.isArray(block.tx) ? block.tx : [];
  let total = 0;
  let count = 0;

  for (const tx of txs) {
    if (!tx || typeof tx !== "object") continue;
    const vouts = Array.isArray((tx as Record<string, unknown>).vout)
      ? ((tx as Record<string, unknown>).vout as unknown[])
      : [];
    for (const vout of vouts) {
      const value = Number((vout as Record<string, unknown>).value);
      if (Number.isFinite(value)) total += value;
      count += 1;
    }
  }

  return { total, count };
}

function extractRewardTxMiner(
  block: Record<string, unknown>,
  coin: CoinId,
): string | undefined {
  const txs = Array.isArray(block.tx) ? block.tx : [];

  for (const tx of txs) {
    if (!tx || typeof tx !== "object") continue;
    const typed = tx as Record<string, unknown>;
    const vins = Array.isArray(typed.vin) ? typed.vin : [];
    const isCoinbase = vins.some(
      (vin) => vin && typeof vin === "object" && "coinbase" in vin,
    );
    const isCoinstake =
      coin === "vericoin" &&
      vins.some(
        (vin) => vin && typeof vin === "object" && "coinstake" in vin,
      );
    if (!isCoinbase && !isCoinstake) continue;

    const vouts = Array.isArray(typed.vout) ? typed.vout : [];
    for (const vout of vouts) {
      const address = voutAddress(vout);
      if (address) return address;
    }
  }

  return undefined;
}

/** Parse `getblock` verbosity 2 into the explorer row shape (mirrors explorer-v2 liveChain). */
export function parseRpcBlock(
  coin: CoinId,
  height: number,
  hash: string,
  block: Record<string, unknown>,
): ExplorerBlock {
  const txs = Array.isArray(block.tx) ? block.tx : [];
  const { total, count } = sumBlockOutputs(block);
  const minerAddress = extractRewardTxMiner(block, coin);
  const output =
    total > 0 ? String(total) : undefined;

  return {
    id: height,
    hash,
    height,
    time:
      typeof block.time === "number"
        ? block.time
        : Math.floor(Date.now() / 1000),
    n_tx:
      block.nTx != null
        ? Number(block.nTx)
        : txs.length > 0
          ? txs.length
          : undefined,
    difficulty:
      block.difficulty != null ? String(block.difficulty) : undefined,
    size: typeof block.size === "number" ? block.size : undefined,
    output_total: output,
    mint: output,
    output_count: count > 0 ? count : undefined,
    miner_address: minerAddress,
  };
}

function isPlaceholderBlockHash(hash: string | undefined): boolean {
  return isPlaceholderTipHash(hash);
}

export function blockNeedsRpcEnrichment(
  block: ExplorerBlock,
  coin: CoinId,
): boolean {
  if (isPlaceholderBlockHash(block.hash)) return true;
  if (block.output_total == null && block.mint == null) return true;
  if (block.size == null || block.difficulty == null) return true;
  if (coin === "verium" && !block.miner_address) return true;
  return false;
}

/** Electrum tip stub or explorer row with no header fields yet. */
export function isIndexingBlockRow(block: ExplorerBlock): boolean {
  if (!isPlaceholderBlockHash(block.hash)) return false;
  return block.time <= 0 || !block.miner_address;
}

/** Full row from local `getblock` via Tauri (works in release builds). */
export async function enrichBlockFromRpc(
  coin: CoinId,
  block: ExplorerBlock,
): Promise<ExplorerBlock | null> {
  if (block.height <= 0) return null;
  try {
    const rows = await fetchLocalBlocksForFeed(coin, [block.height]);
    return rows[0] ?? null;
  } catch {
    return null;
  }
}

export async function enrichBlocksFromRpc(
  coin: CoinId,
  blocks: ExplorerBlock[],
): Promise<ExplorerBlock[]> {
  const heights = [
    ...new Set(
      blocks
        .filter((block) => blockNeedsRpcEnrichment(block, coin))
        .map((block) => block.height)
        .filter((height) => height > 0),
    ),
  ];
  if (heights.length === 0) return [];

  try {
    return await fetchLocalBlocksForFeed(coin, heights);
  } catch {
    return [];
  }
}

/** Fill recent-blocks rows from explorer block detail (light wallet). */
export async function enrichBlocksFromExplorer(
  coin: CoinId,
  blocks: ExplorerBlock[],
): Promise<ExplorerBlock[]> {
  const heights = [
    ...new Set(
      blocks
        .filter((block) => blockNeedsRpcEnrichment(block, coin))
        .map((block) => block.height)
        .filter((height) => height > 0),
    ),
  ];
  if (heights.length === 0) return [];

  try {
    return await fetchExplorerBlocksForFeed(coin, heights);
  } catch {
    return [];
  }
}

async function resolveBlockHash(
  coin: CoinId,
  event: WalletRewardEvent,
): Promise<string | undefined> {
  if (event.blockhash) return event.blockhash;
  if (event.height <= 0) return undefined;
  try {
    const hash = await rpcRaw(coin, "getblockhash", [event.height]);
    return typeof hash === "string" ? hash : undefined;
  } catch {
    return undefined;
  }
}

/** Build a recent-blocks row from the local node as soon as the wallet sees the reward. */
export async function blockRowFromRewardEvent(
  coin: CoinId,
  event: WalletRewardEvent,
): Promise<ExplorerBlock | null> {
  if (event.height <= 0) return null;

  const hash = await resolveBlockHash(coin, event);
  const reward =
    event.amount != null && Number.isFinite(event.amount)
      ? String(event.amount)
      : undefined;

  if (!hash) {
    return {
      id: event.height,
      hash: event.txid ?? `local-${event.height}`,
      height: event.height,
      time: event.blocktime ?? Math.floor(Date.now() / 1000),
      mint: reward,
      output_total: reward,
      miner_address: event.address,
    };
  }

  const enriched = await enrichBlockFromRpc(coin, {
    id: event.height,
    hash,
    height: event.height,
    time: event.blocktime ?? 0,
  });

  if (!enriched) {
    return {
      id: event.height,
      hash,
      height: event.height,
      time: event.blocktime ?? Math.floor(Date.now() / 1000),
      mint: reward,
      output_total: reward,
      miner_address: event.address,
    };
  }

  return {
    ...enriched,
    mint: reward ?? enriched.mint,
    output_total: reward ?? enriched.output_total,
    miner_address: event.address ?? enriched.miner_address,
  };
}

/** @deprecated Use `blockRowFromRewardEvent`. */
export const blockRowFromMinedEvent = blockRowFromRewardEvent;

/** Load one main-chain block from the local node with full table fields. */
export async function fetchLocalBlockRow(
  coin: CoinId,
  height: number,
): Promise<ExplorerBlock | null> {
  return enrichBlockFromRpc(coin, {
    id: height,
    hash: `local-pending-${height}`,
    height,
    time: 0,
  });
}

/**
 * Backfill blocks the local node has but the explorer index has not caught up to
 * yet (common when the wallet is +N vs explorer).
 */
export async function fetchLocalBlocksAbove(
  coin: CoinId,
  aboveHeight: number,
  tipHeight: number,
  limit = 12,
): Promise<ExplorerBlock[]> {
  if (tipHeight <= aboveHeight) return [];

  const rows: ExplorerBlock[] = [];
  for (let height = tipHeight; height > aboveHeight && rows.length < limit; height -= 1) {
    const row = await fetchLocalBlockRow(coin, height);
    if (row) rows.push(row);
  }
  return rows;
}

/**
 * Max placeholder rows when the wallet tip is ahead of the explorer index.
 * Without a cap, a 100k+ height gap materializes one string per height in the
 * WebView heap (see heap snapshot analysis).
 */
export const MAX_PENDING_BLOCKS_ABOVE = 24;

/** Instant rows for heights above the explorer index (filled in via RPC after). */
export function buildPendingBlocksAbove(
  aboveHeight: number,
  tipHeight: number,
  tipHash: string | undefined,
  tipTime: number,
  knownHeights: ReadonlySet<number>,
  limit = MAX_PENDING_BLOCKS_ABOVE,
): ExplorerBlock[] {
  if (tipHeight <= aboveHeight) return [];

  const rows: ExplorerBlock[] = [];
  for (let height = tipHeight; height > aboveHeight && rows.length < limit; height -= 1) {
    if (knownHeights.has(height)) continue;
    rows.push({
      id: height,
      hash:
        height === tipHeight && tipHash
          ? tipHash
          : `local-pending-${height}`,
      height,
      time: height === tipHeight && tipTime > 0 ? tipTime : 0,
    });
  }
  return rows;
}
