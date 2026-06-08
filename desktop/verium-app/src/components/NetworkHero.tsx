import type { CoinId, CoinProfile } from "@/lib/coin/profile";
import { AnimatedBlockNumber } from "@/components/AnimatedBlockNumber";
import { Badge } from "@/components/ui/Badge";
import { ExplorerLink } from "@/components/ExplorerLink";
import { blocksBehindNetwork } from "@/lib/bootstrap-policy";
import type { ExplorerStats } from "@/lib/explorer-api";
import { networkHashToKhm, resolveBlockTimeMinutes } from "@/lib/mining-revenue";
import type {
  BlockchainInfo,
  NetworkInfo,
  VericoinMiningInfo,
} from "@/lib/rpc/client";
import {
  mergeStakingNetworkKpis,
  networkCoinsStakingPercent,
} from "@/lib/staking-stats";
import { cn, formatNumber } from "@/lib/utils";
import { Clock3, Radio } from "lucide-react";

export interface NetworkHeroProps {
  profile: CoinProfile;
  coin: CoinId;
  localBlocks?: number;
  headerHeight?: number;
  networkTip?: number;
  peerCount: number;
  network?: NetworkInfo | null;
  blockchain?: BlockchainInfo | null;
  explorer?: ExplorerStats | null;
  explorerError?: boolean;
  localHashrate?: number;
  vrcMining?: VericoinMiningInfo | null;
  networkActive?: boolean;
  chainSynced: boolean;
  ibd?: boolean;
}

function formatUsd(value?: number | null): string {
  if (value == null || !Number.isFinite(value)) return "—";
  if (value >= 1_000_000) return `$${formatNumber(value / 1_000_000, 2)}M`;
  if (value >= 1_000) return `$${formatNumber(value / 1_000, 2)}K`;
  return `$${formatNumber(value, 4)}`;
}

function NetMetric({ label, value }: { label: string; value: string }) {
  return (
    <div className="min-w-0 py-1.5 xl:border-l xl:border-border/40 xl:py-1 xl:pl-4 xl:first:border-l-0 xl:first:pl-0">
      <div className="text-[11px] font-semibold uppercase tracking-wide text-fg-subtle sm:text-[10px]">
        {label}
      </div>
      <div className="mt-1 truncate text-base font-semibold tabular-nums text-fg sm:mt-0.5 sm:text-sm">
        {value}
      </div>
    </div>
  );
}

function NodeStat({
  label,
  value,
  sub,
}: {
  label: string;
  value: string;
  sub?: string;
}) {
  return (
    <div className="min-w-0 rounded-lg bg-bg-subtle/35 px-3 py-2.5 ring-1 ring-inset ring-border/45 sm:px-3.5 sm:py-3">
      <div className="text-[10px] font-semibold uppercase tracking-wide text-fg-subtle">
        {label}
      </div>
      <div className="mt-0.5 truncate text-lg font-semibold tabular-nums text-fg sm:text-base">
        {value}
      </div>
      {sub ? (
        <div className="mt-0.5 truncate text-[10px] text-fg-subtle">{sub}</div>
      ) : null}
    </div>
  );
}

export function NetworkHero({
  profile,
  coin,
  localBlocks,
  headerHeight = 0,
  networkTip,
  peerCount,
  network,
  blockchain,
  explorer,
  explorerError,
  localHashrate,
  vrcMining,
  networkActive,
  chainSynced,
  ibd,
}: NetworkHeroProps) {
  const behind = blocksBehindNetwork(localBlocks, networkTip ?? headerHeight);
  const lag = Math.max(0, headerHeight - (localBlocks ?? 0));
  const syncTarget = Math.max(headerHeight, networkTip ?? 0);
  const showSyncBar =
    !chainSynced &&
    syncTarget > (localBlocks ?? 0) &&
    localBlocks != null;
  const syncPct =
    showSyncBar && syncTarget > 0
      ? Math.min(100, ((localBlocks ?? 0) / syncTarget) * 100)
      : 100;

  const heightDelta =
    explorer?.height != null && localBlocks != null
      ? localBlocks - explorer.height
      : undefined;
  const matchesExplorer =
    heightDelta != null && Math.abs(heightDelta) <= 1;

  const subversion = network?.subversion
    ?.replace(/^\//, "")
    .replace(/\/$/, "");

  const blockTimeMin = resolveBlockTimeMinutes(explorer, null);

  const networkMetrics =
    coin === "verium" ? (
      <>
        <NetMetric
          label="Network hashrate"
          value={
            explorer?.network_hash != null
              ? `${formatNumber(networkHashToKhm(explorer.network_hash), 2)} kH/m`
              : localHashrate != null
                ? `${formatNumber(localHashrate, 0)} H/m`
                : "—"
          }
        />
        <NetMetric
          label="Difficulty"
          value={
            explorer?.difficulty != null
              ? formatNumber(explorer.difficulty, 7)
              : blockchain?.difficulty != null
                ? formatNumber(blockchain.difficulty, 7)
                : "—"
          }
        />
        <NetMetric
          label="Block reward"
          value={
            explorer?.block_reward != null
              ? `${formatNumber(explorer.block_reward, 4)} VRM`
              : "—"
          }
        />
        <NetMetric
          label="Supply"
          value={
            explorer?.supply != null
              ? `${formatNumber(explorer.supply, 2)} VRM`
              : "—"
          }
        />
        <NetMetric label="VRM price" value={formatUsd(explorer?.price_usd)} />
        <NetMetric
          label="Avg block time"
          value={
            blockTimeMin != null ? `${formatNumber(blockTimeMin, 2)} min` : "—"
          }
        />
        <NetMetric
          label="Mempool"
          value={
            explorer?.pooled_tx != null
              ? formatNumber(explorer.pooled_tx, 0)
              : "—"
          }
        />
        <NetMetric
          label="Market cap"
          value={formatUsd(explorer?.market_cap_usd)}
        />
        <NetMetric
          label="24h volume"
          value={formatUsd(explorer?.volume_24h_usd)}
        />
      </>
    ) : (
      (() => {
        const staking = mergeStakingNetworkKpis(vrcMining, explorer);
        const stakePct = networkCoinsStakingPercent(staking.netStakeWeight);
        return (
          <>
            <NetMetric
              label="Interest rate"
              value={
                staking.interestRate != null
                  ? `${formatNumber(staking.interestRate, 2)}%`
                  : "—"
              }
            />
            <NetMetric
              label="Network staked"
              value={
                stakePct != null ? `${formatNumber(stakePct, 2)}%` : "—"
              }
            />
            <NetMetric
              label="PoS difficulty"
              value={
                staking.posDifficulty != null
                  ? formatNumber(staking.posDifficulty, 4)
                  : "—"
              }
            />
            <NetMetric
              label="Supply"
              value={
                staking.supply != null
                  ? `${formatNumber(staking.supply, 2)} VRC`
                  : "—"
              }
            />
            <NetMetric label="VRC price" value={formatUsd(explorer?.price_usd)} />
            <NetMetric
              label="Avg block time"
              value={
                blockTimeMin != null
                  ? `${formatNumber(blockTimeMin, 2)} min`
                  : "—"
              }
            />
            <NetMetric
              label="Mempool"
              value={
                explorer?.pooled_tx != null
                  ? formatNumber(explorer.pooled_tx, 0)
                  : "—"
              }
            />
            <NetMetric
              label="Market cap"
              value={formatUsd(explorer?.market_cap_usd)}
            />
            <NetMetric
              label="24h volume"
              value={formatUsd(explorer?.volume_24h_usd)}
            />
          </>
        );
      })()
    );

  return (
    <section
      aria-label={`${profile.displayName} network`}
      className="relative min-w-0 overflow-hidden rounded-xl border border-border bg-bg-panel shadow-sm"
    >
      <div
        className="pointer-events-none absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-accent/35 to-transparent"
        aria-hidden
      />
      <div
        className="pointer-events-none absolute -right-24 -top-24 h-48 w-48 rounded-full bg-accent/[0.04] blur-3xl"
        aria-hidden
      />

      <div className="relative min-w-0 p-5 sm:p-6 xl:p-6">
        <header className="flex min-w-0 flex-wrap items-center gap-2">
          <Radio className="h-4 w-4 shrink-0 text-accent" aria-hidden />
          <h2 className="text-base font-semibold text-fg sm:text-lg">
            {profile.displayName} network
          </h2>
          <Badge tone={networkActive ? "success" : "neutral"}>
            {networkActive ? "P2P active" : "P2P idle"}
          </Badge>
          {chainSynced ? (
            <Badge tone="success">At chain tip</Badge>
          ) : ibd ? (
            <Badge tone="warning">Syncing</Badge>
          ) : (
            <Badge tone="warning">Catching up</Badge>
          )}
          {explorer && !explorerError && !matchesExplorer && heightDelta != null && (
            <Badge tone="neutral">
              {heightDelta >= 0 ? "+" : ""}
              {formatNumber(heightDelta, 0)} vs explorer
            </Badge>
          )}
          <div className="ml-auto">
            <ExplorerLink coin={coin} target={{ kind: "home" }} label="Explorer" />
          </div>
        </header>

        <p className="mt-1.5 text-sm text-fg-muted">
          Node sync, peers, and public {profile.symbol} chain data.
        </p>

        <div className="mt-5 grid min-w-0 gap-4 lg:grid-cols-12 lg:gap-5 xl:mt-6 xl:gap-6">
          <div className="min-w-0 rounded-xl bg-bg-subtle/25 p-4 ring-1 ring-inset ring-border/40 sm:p-5 lg:col-span-5 xl:col-span-4">
            <div className="text-[11px] font-semibold uppercase tracking-wide text-fg-subtle">
              Local block height
            </div>
            <div className="mt-2">
              <div
                className={cn(
                  "text-[clamp(1.75rem,3.5vw+0.5rem,2.75rem)] font-bold tabular-nums leading-none tracking-tight",
                  localBlocks != null ? "text-fg" : "text-fg-muted",
                )}
              >
                {localBlocks != null ? (
                  <ExplorerLink
                    coin={profile.id}
                    target={{ kind: "block", hashOrHeight: localBlocks }}
                    label={
                      <AnimatedBlockNumber
                        value={localBlocks}
                        className="text-[clamp(1.75rem,3.5vw+0.5rem,2.75rem)] font-bold leading-none"
                      />
                    }
                    showIcon={false}
                    className="font-bold text-fg no-underline hover:text-accent"
                  />
                ) : (
                  "—"
                )}
              </div>
            </div>
            <p className="mt-2 text-sm text-fg-muted">
              Headers{" "}
              <span className="font-medium tabular-nums text-fg">
                {headerHeight > 0 ? formatNumber(headerHeight) : "—"}
              </span>
              {networkTip != null && networkTip > headerHeight && (
                <>
                  {" "}
                  · tip ~{formatNumber(networkTip)}
                </>
              )}
            </p>
            {behind != null && behind > 0 && (
              <p className="mt-1 text-xs text-fg-subtle">
                ~{formatNumber(behind)} blocks behind network tip
              </p>
            )}
            {explorer?.blocks_per_hour != null && (
              <p className="mt-2 inline-flex items-center gap-1.5 text-xs text-fg-subtle">
                <Clock3 className="h-3.5 w-3.5 opacity-70" aria-hidden />
                {formatNumber(explorer.blocks_per_hour, 2)} blocks / hour
              </p>
            )}
            {showSyncBar && (
              <div className="mt-4">
                <div className="mb-1 flex justify-between text-xs text-fg-muted">
                  <span>Sync progress</span>
                  <span className="tabular-nums">
                    {formatNumber(syncPct, 1)}%
                  </span>
                </div>
                <div className="h-1.5 overflow-hidden rounded-full bg-border/80">
                  <div
                    className="h-full rounded-full bg-accent transition-[width] duration-500"
                    style={{ width: `${syncPct}%` }}
                  />
                </div>
              </div>
            )}
          </div>

          <div className="grid min-w-0 grid-cols-2 gap-3 lg:col-span-7 lg:grid-cols-2 xl:col-span-8">
            <NodeStat label="Peers" value={formatNumber(peerCount)} />
            <NodeStat
              label="Protocol"
              value={
                network?.protocolversion != null
                  ? formatNumber(network.protocolversion, 0)
                  : "—"
              }
              sub={subversion}
            />
            <NodeStat
              label="Header lag"
              value={headerHeight > 0 ? formatNumber(lag) : "—"}
              sub={
                headerHeight > 0
                  ? `${formatNumber(localBlocks ?? 0)} / ${formatNumber(headerHeight)} blocks`
                  : undefined
              }
            />
            <NodeStat
              label="Explorer height"
              value={
                explorer?.height != null
                  ? formatNumber(explorer.height, 0)
                  : explorerError
                    ? "Unavailable"
                    : "—"
              }
              sub={
                matchesExplorer
                  ? "Matches your node"
                  : heightDelta != null
                    ? `${heightDelta >= 0 ? "+" : ""}${formatNumber(heightDelta, 0)} delta`
                    : undefined
              }
            />
          </div>
        </div>

        <footer className="mt-6 border-t border-border/50 pt-5">
          <div className="mb-3 text-[11px] font-semibold uppercase tracking-wider text-fg-subtle sm:text-[10px]">
            {explorerError
              ? "Network (local node)"
              : `${profile.displayName} network`}
          </div>
          <div className="grid grid-cols-2 gap-x-4 gap-y-4 sm:grid-cols-3 lg:grid-cols-3 xl:grid-cols-5 xl:gap-y-0">
            {networkMetrics}
          </div>
        </footer>
      </div>
    </section>
  );
}
