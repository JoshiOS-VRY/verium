import { Loader2, Pickaxe, Trophy } from "lucide-react";
import { ExplorerLink } from "@/components/ExplorerLink";
import { AnimatedBlockNumber } from "@/components/AnimatedBlockNumber";
import {
  YouMinedBadge,
  youMinedRowClassName,
} from "@/components/YouMinedCelebration";
import {
  YouStakedBadge,
  youStakedRowClassName,
} from "@/components/YouStakedCelebration";
import {
  isBlockMinedByWallet,
  useWalletMiningContext,
} from "@/hooks/useWalletMiningContext";
import {
  isBlockStakedByWallet,
  useWalletStakingContext,
} from "@/hooks/useWalletStakingContext";
import {
  isFreshMinedBlock,
} from "@/components/YouMinedCelebration";
import {
  isFreshStakedReward,
} from "@/components/YouStakedCelebration";
import type { ExplorerBlock } from "@/lib/explorer-api";
import { isIndexingBlockRow } from "@/lib/local-recent-block";
import {
  isVeriumPoolMinerAddress,
  resolveVeriumMinerExplorerAddress,
  VERIUM_POOL_DISPLAY_NAME,
} from "@/lib/verium-pool-labels";
import type { CoinId } from "@/lib/coin/profile";
import { formatCoinAmount } from "@/lib/units";
import { cn, formatBlockAge, formatNumber } from "@/lib/utils";

export function formatBlockDifficulty(value?: string): string {
  if (!value) return "—";
  const n = Number(value);
  if (!Number.isFinite(n)) return value;
  if (n < 0.00001) return n.toExponential(2);
  return formatNumber(n, 7);
}

export function formatBlockOutput(value: string | undefined, coin: CoinId): string {
  if (!value) return "—";
  const n = Number(value);
  if (!Number.isFinite(n)) return value;
  return formatCoinAmount(n, coin, 4);
}

export interface RecentBlockRowModel {
  block: ExplorerBlock;
  isTip: boolean;
  indexing: boolean;
  isYours: boolean;
  isFresh: boolean;
  reward: string;
  isEntering: boolean;
  isNudging: boolean;
  rowClassName: string;
  poolMiner: boolean;
  minerLinkAddress: string | null;
}

export function buildRecentBlockRowModel(
  block: ExplorerBlock,
  {
    coin,
    tipHeight,
    enteringHash,
    nudgeOthers,
    miningCtx,
    stakingCtx,
  }: {
    coin: CoinId;
    tipHeight: number | undefined;
    enteringHash: string | null;
    nudgeOthers: boolean;
    miningCtx: ReturnType<typeof useWalletMiningContext>;
    stakingCtx: ReturnType<typeof useWalletStakingContext>;
  },
): RecentBlockRowModel {
  const isVerium = coin === "verium";
  const isTip = tipHeight === block.height;
  const indexing = isIndexingBlockRow(block);
  const isYours = isVerium
    ? isBlockMinedByWallet(block, miningCtx)
    : isBlockStakedByWallet(block, stakingCtx);
  const isFresh = isVerium
    ? isYours && isFreshMinedBlock(block.time)
    : isYours && isFreshStakedReward(block.time);
  const reward = formatBlockOutput(block.output_total ?? block.mint, coin);
  const isEntering = enteringHash === block.hash;
  const isNudging = nudgeOthers && !isEntering;
  const rowClassName = isVerium
    ? youMinedRowClassName({ isYours, isFresh, isTip })
    : youStakedRowClassName({ isYours, isFresh, isTip });
  const poolMiner =
    isVerium &&
    Boolean(block.miner_address) &&
    isVeriumPoolMinerAddress(block.miner_address);
  const minerLinkAddress = block.miner_address
    ? (resolveVeriumMinerExplorerAddress(block.miner_address) ??
      block.miner_address)
    : null;

  return {
    block,
    isTip,
    indexing,
    isYours,
    isFresh,
    reward,
    isEntering,
    isNudging,
    rowClassName,
    poolMiner,
    minerLinkAddress,
  };
}

function BlockMinerLabel({
  coin,
  model,
  isVerium,
}: {
  coin: CoinId;
  model: RecentBlockRowModel;
  isVerium: boolean;
}) {
  const { block, indexing, isYours, poolMiner, minerLinkAddress } = model;

  if (indexing) return <span className="text-fg-subtle">—</span>;

  if (isYours) {
    if (block.miner_address) {
      return (
        <ExplorerLink
          coin={coin}
          target={{ kind: "address", address: block.miner_address }}
          label="Your Wallet"
          className={cn(
            "inline-flex max-w-full items-center gap-1.5 truncate font-medium",
            isVerium
              ? "text-success hover:text-success"
              : "text-accent hover:text-accent",
          )}
        />
      );
    }
    return (
      <span
        className={cn(
          "inline-flex items-center gap-1.5 font-medium",
          isVerium ? "text-success" : "text-accent",
        )}
      >
        {isVerium ? (
          <Pickaxe className="h-3 w-3 shrink-0 opacity-80" aria-hidden />
        ) : (
          <Trophy className="h-3 w-3 shrink-0 opacity-80" aria-hidden />
        )}
        You
      </span>
    );
  }

  if (minerLinkAddress) {
    if (poolMiner) {
      return (
        <ExplorerLink
          coin={coin}
          target={{ kind: "address", address: minerLinkAddress }}
          label={VERIUM_POOL_DISPLAY_NAME}
          showIcon={false}
          title={minerLinkAddress}
          className="inline-flex max-w-full shrink-0 items-center rounded-full border border-border bg-bg-subtle px-2 py-0.5 text-xs font-medium text-fg-muted no-underline hover:border-border hover:bg-bg-subtle hover:text-fg"
        />
      );
    }
    return (
      <ExplorerLink
        coin={coin}
        target={{ kind: "address", address: minerLinkAddress }}
        label={block.miner_address}
      />
    );
  }

  return <span className="text-fg-subtle">—</span>;
}

function BlockHeightBadge({
  model,
  isVerium,
}: {
  model: RecentBlockRowModel;
  isVerium: boolean;
}) {
  const { block, isTip, isYours, isFresh, isEntering } = model;

  return (
    <div className="flex flex-wrap items-center gap-2">
      <span
        className={cn(
          "inline-flex items-center rounded-md px-1.5 py-0.5 tabular-nums",
          isYours &&
            (isVerium
              ? "bg-success/12 font-semibold text-success"
              : "bg-accent/12 font-semibold text-accent"),
          !isYours && (isTip ? "font-medium text-accent" : "text-fg"),
        )}
      >
        <AnimatedBlockNumber
          value={block.height}
          forceSpring={isEntering}
          animateOnIncrease={false}
        />
      </span>
      {isYours &&
        (isVerium ? (
          <YouMinedBadge fresh={isFresh} />
        ) : (
          <YouStakedBadge fresh={isFresh} />
        ))}
    </div>
  );
}

function BlockTimeLabel({
  model,
  ageTick,
}: {
  model: RecentBlockRowModel;
  ageTick: number;
}) {
  const { block, indexing, isYours } = model;

  if (indexing) {
    return (
      <span className="inline-flex items-center gap-1 text-fg-subtle">
        <Loader2 className="h-3 w-3 animate-spin" aria-hidden />
        Indexing…
      </span>
    );
  }

  return (
    <span
      className={cn(
        "tabular-nums",
        isYours ? "font-medium text-fg" : "text-fg-muted",
      )}
    >
      {block.time > 0 ? formatBlockAge(block.time, ageTick) : "—"}
    </span>
  );
}

export function RecentBlockTableRow({
  coin,
  model,
  isDashboard,
  ageTick,
}: {
  coin: CoinId;
  model: RecentBlockRowModel;
  isDashboard: boolean;
  ageTick: number;
}) {
  const isVerium = coin === "verium";
  const { block, indexing, isYours, reward, isEntering, isNudging, rowClassName } =
    model;

  return (
    <tr
      className={cn(
        "border-t border-border transition-[background-color,box-shadow]",
        rowClassName,
        isEntering && "block-row-enter",
        isNudging && "block-row-nudge",
      )}
    >
      <td className="px-4 py-2.5 tabular-nums">
        <BlockHeightBadge model={model} isVerium={isVerium} />
      </td>
      <td className="px-4 py-2.5 text-right text-xs">
        <BlockTimeLabel model={model} ageTick={ageTick} />
      </td>
      <td
        className={cn(
          "px-4 py-2.5 text-right tabular-nums",
          isYours ? "text-fg" : "text-fg-muted",
        )}
      >
        {indexing ? "—" : (block.n_tx ?? "—")}
      </td>
      <td
        className={cn(
          "px-4 py-2.5 text-right text-xs tabular-nums",
          isYours &&
            (isVerium
              ? "font-semibold text-success"
              : "font-semibold text-accent"),
        )}
      >
        {indexing ? "—" : reward}
      </td>
      {isDashboard && (
        <>
          <td className="hidden px-4 py-2.5 text-right text-xs tabular-nums text-fg-muted sm:table-cell">
            {!indexing && block.size != null
              ? `${formatNumber(block.size, 0)} B`
              : "—"}
          </td>
          <td className="hidden px-4 py-2.5 text-right text-xs tabular-nums text-fg-muted md:table-cell">
            {indexing ? "—" : formatBlockDifficulty(block.difficulty)}
          </td>
        </>
      )}
      <td className="max-w-[180px] px-4 py-2.5 text-xs">
        <BlockMinerLabel coin={coin} model={model} isVerium={isVerium} />
      </td>
    </tr>
  );
}

export function RecentBlockCard({
  coin,
  model,
  isDashboard,
  ageTick,
}: {
  coin: CoinId;
  model: RecentBlockRowModel;
  isDashboard: boolean;
  ageTick: number;
}) {
  const isVerium = coin === "verium";
  const { block, indexing, isYours, reward, isEntering, isNudging, rowClassName } =
    model;

  return (
    <article
      className={cn(
        "recent-block-card rounded-xl border border-border bg-bg-subtle/60 p-3 transition-[background-color,box-shadow,transform]",
        rowClassName,
        isEntering && "block-row-enter",
        isNudging && "block-row-nudge",
        isYours &&
          (isVerium
            ? "border-success/25 bg-success/5"
            : "border-accent/25 bg-accent/5"),
      )}
    >
      <div className="flex items-start justify-between gap-3">
        <BlockHeightBadge model={model} isVerium={isVerium} />
        <div className="shrink-0 text-right text-xs">
          <BlockTimeLabel model={model} ageTick={ageTick} />
        </div>
      </div>

      <dl className="mt-3 grid grid-cols-2 gap-x-3 gap-y-2 text-xs">
        <div>
          <dt className="text-fg-subtle">Output</dt>
          <dd
            className={cn(
              "mt-0.5 tabular-nums font-medium",
              isYours &&
                (isVerium ? "text-success" : "text-accent"),
              !isYours && "text-fg",
            )}
          >
            {indexing ? "—" : reward}
          </dd>
        </div>
        <div>
          <dt className="text-fg-subtle">Transactions</dt>
          <dd
            className={cn(
              "mt-0.5 tabular-nums",
              isYours ? "text-fg" : "text-fg-muted",
            )}
          >
            {indexing ? "—" : (block.n_tx ?? "—")}
          </dd>
        </div>
        {isDashboard && (
          <>
            <div>
              <dt className="text-fg-subtle">Size</dt>
              <dd className="mt-0.5 tabular-nums text-fg-muted">
                {!indexing && block.size != null
                  ? `${formatNumber(block.size, 0)} B`
                  : "—"}
              </dd>
            </div>
            <div>
              <dt className="text-fg-subtle">Difficulty</dt>
              <dd className="mt-0.5 tabular-nums text-fg-muted">
                {indexing ? "—" : formatBlockDifficulty(block.difficulty)}
              </dd>
            </div>
          </>
        )}
      </dl>

      <div className="mt-3 border-t border-border/60 pt-2.5">
        <p className="text-[10px] font-medium uppercase tracking-wide text-fg-subtle">
          {isVerium ? "Mined by" : "Staked by"}
        </p>
        <div className="mt-1 text-xs">
          <BlockMinerLabel coin={coin} model={model} isVerium={isVerium} />
        </div>
      </div>

      <div className="mt-2 flex justify-end">
        <ExplorerLink
          coin={coin}
          target={{ kind: "block", hashOrHeight: block.hash || block.height }}
          label="View block"
          className="text-[11px]"
        />
      </div>
    </article>
  );
}
