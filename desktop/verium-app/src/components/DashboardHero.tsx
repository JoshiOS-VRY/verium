import { type ReactNode } from 'react';
import { Link } from 'react-router-dom';
import { Coins, Loader2, TrendingUp, Users, Wallet } from 'lucide-react';
import { ExplorerLink } from '@/components/ExplorerLink';
import { MiningPickaxeAnimation } from '@/components/MiningPickaxeAnimation';
import { MinerBootBadge } from '@/components/MinerBootIndicator';
import { getCoinProfile, type CoinId, type CoinProfile } from '@/lib/coin/profile';
import { useDashboardData } from '@/hooks/useDashboardData';
import { heroStatusPillLabel, heroStatusPillShowsPulse } from '@/lib/node/dashboard-activity';
import {
  buildNetworkStats,
  estimateDailyMining,
  networkHashToKhm,
  networkSharePercent,
  resolveBlockTimeMinutes,
} from '@/lib/mining-revenue';
import { isMinerBooting } from '@/lib/mining-boot';
import {
  mergeStakingNetworkKpis,
  networkCoinsStakingPercent,
  walletStakeSharePercent,
} from '@/lib/staking-stats';
import { useExplorerQueriesEnabled } from '@/lib/network-mode';
import { AnimatedHashrate } from '@/components/AnimatedHashrate';
import { cn, formatNumber } from '@/lib/utils';
import { formatCoinAmount } from '@/lib/units';
import { lockedWalletBalanceClass } from '@/lib/wallet-unlock';

type DashboardData = ReturnType<typeof useDashboardData>;

function formatUsd(value?: number | null): string {
  if (value === undefined || value === null) return '—';
  if (value >= 1_000_000) return `$${formatNumber(value / 1_000_000, 2)}M`;
  if (value >= 1_000) return `$${formatNumber(value / 1_000, 2)}K`;
  return `$${formatNumber(value, 4)}`;
}

function StatusPill({
  children,
  tone = 'neutral',
  loading = false,
}: {
  children: ReactNode;
  tone?: 'neutral' | 'success' | 'accent';
  loading?: boolean;
}) {
  return (
    <div
      className={cn(
        'inline-flex max-w-full items-center gap-1.5 rounded-full border px-2.5 py-1 text-[11px] font-semibold shadow-sm',
        tone === 'success' && 'border-success/25 bg-success/10 text-success ring-1 ring-success/10',
        tone === 'accent' && 'border-accent/25 bg-accent/10 text-accent ring-1 ring-accent/10',
        tone === 'neutral' && 'border-border/80 bg-bg-subtle/80 text-fg-muted'
      )}
    >
      {loading && <Loader2 className="h-3 w-3 shrink-0 animate-spin" aria-hidden />}
      {children}
    </div>
  );
}

function MiniStat({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className="min-w-0">
      <div className="text-[11px] font-medium uppercase tracking-wide text-fg-subtle sm:text-[10px]">
        {label}
      </div>
      <div className="mt-1 truncate text-base font-semibold tabular-nums text-fg sm:mt-0.5 sm:text-sm">
        {value}
      </div>
    </div>
  );
}

function HeroSection({
  title,
  icon,
  action,
  href,
  children,
  className,
}: {
  title: string;
  icon: ReactNode;
  action?: ReactNode;
  href?: string;
  children: ReactNode;
  className?: string;
}) {
  const body = (
    <div
      className={cn(
        'flex h-full min-w-0 flex-col rounded-xl bg-bg-subtle/30 p-4 ring-1 ring-inset ring-border/45 sm:p-4 md:p-3.5 xl:p-4',
        href &&
          'transition-colors hover:bg-bg-subtle/50 hover:ring-border/70 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent'
      )}
    >
      <div className="mb-3 flex min-w-0 items-center justify-between gap-2 md:mb-2.5">
        <div className="flex min-w-0 items-center gap-2 text-sm font-semibold text-fg">
          <span className="shrink-0 text-accent [&>svg]:h-4 [&>svg]:w-4">{icon}</span>
          <span className="truncate">{title}</span>
        </div>
        {action}
      </div>
      <div className="grid flex-1 grid-cols-2 gap-x-4 gap-y-3 sm:gap-x-5 md:gap-x-3 md:gap-y-2.5 xl:gap-x-4 xl:gap-y-3">
        {children}
      </div>
    </div>
  );

  if (href) {
    return (
      <Link
        to={href}
        aria-label={`Open ${title.toLowerCase()}`}
        className={cn('block h-full min-w-0', className)}
      >
        {body}
      </Link>
    );
  }

  return <div className={cn('h-full min-w-0', className)}>{body}</div>;
}

function SyncProgressBar({
  localBlocks,
  syncTarget,
  behind,
}: {
  localBlocks?: number;
  syncTarget: number;
  behind?: number | null;
}) {
  const progress =
    localBlocks != null && syncTarget > 0
      ? Math.min(100, Math.round((localBlocks / syncTarget) * 100))
      : 0;

  return (
    <div className="mt-3 w-full min-w-0 space-y-2">
      <div className="flex items-center justify-between gap-3 text-[11px] text-fg-muted">
        <span>
          of ~{formatNumber(syncTarget)} network tip
          {behind != null && behind > 0 && <> · ~{formatNumber(behind, 0)} blocks behind</>}
        </span>
        <span className="shrink-0 font-semibold tabular-nums">{progress}%</span>
      </div>
      <div
        className="h-1.5 overflow-hidden rounded-full bg-border/80"
        role="progressbar"
        aria-valuenow={progress}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-label="Chain sync progress"
      >
        <div
          className="h-full rounded-full bg-gradient-to-r from-accent/70 to-accent transition-[width] duration-700 ease-out"
          style={{ width: `${progress}%` }}
        />
      </div>
    </div>
  );
}

function NetworkMetric({ label, value }: { label: string; value: string }) {
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

function HeroPanel({
  profile,
  statusRow,
  localBlocks,
  activity,
  synced,
  syncTarget,
  behind,
  detailColumns,
  networkMetrics,
}: {
  profile: CoinProfile;
  statusRow: ReactNode;
  localBlocks?: number;
  activity: DashboardData['activity'];
  synced: boolean;
  syncTarget?: number;
  behind?: number | null;
  detailColumns: ReactNode;
  networkMetrics: ReactNode;
}) {
  const showSyncProgress =
    !synced && syncTarget != null && syncTarget > (localBlocks ?? 0) && localBlocks != null;

  return (
    <section
      aria-label={`${profile.displayName} dashboard`}
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
        <header className="flex min-w-0 flex-wrap items-center gap-2.5">{statusRow}</header>

        {activity.kind !== 'ready' && (
          <p className="mt-3 text-xs leading-relaxed text-fg-muted">{activity.title}</p>
        )}

        {showSyncProgress && (
          <div className="mt-4">
            <SyncProgressBar localBlocks={localBlocks} syncTarget={syncTarget} behind={behind} />
          </div>
        )}

        <div className="mt-5 grid min-w-0 grid-cols-1 gap-4 lg:grid-cols-2 lg:gap-3 xl:mt-6 xl:grid-cols-3 xl:gap-4">
          {detailColumns}
        </div>

        <footer className="mt-6 border-t border-border/50 pt-5 xl:mt-6 xl:pt-5">
          <div className="mb-3 text-[11px] font-semibold uppercase tracking-wider text-fg-subtle sm:mb-2.5 sm:text-[10px]">
            {profile.displayName} network
          </div>
          <div className="grid grid-cols-2 gap-x-4 gap-y-4 sm:gap-x-5 md:grid-cols-3 xl:grid-cols-5 xl:gap-y-0">
            {networkMetrics}
          </div>
        </footer>
      </div>
    </section>
  );
}

function buildHeroStatusRow(data: DashboardData): ReactNode {
  if (data.isLight) {
    const online = data.connected;
    return (
      <>
        <StatusPill tone={online ? 'success' : 'accent'}>
          {online && (
            <span className="inline-block h-1.5 w-1.5 animate-pulse rounded-full bg-success" />
          )}
          {data.connected ? 'Light wallet online' : 'Light wallet offline'}
        </StatusPill>
        <StatusPill tone="neutral">Light wallet</StatusPill>
      </>
    );
  }

  const { activity, synced, blockchain } = data;

  const pillLoading = heroStatusPillShowsPulse(activity);
  const pillTone = synced && activity.kind === 'ready' ? 'success' : 'accent';

  return (
    <>
      <StatusPill tone={pillTone} loading={pillLoading}>
        {synced && activity.kind === 'ready' && (
          <span className="inline-block h-1.5 w-1.5 animate-pulse rounded-full bg-success" />
        )}
        {heroStatusPillLabel(activity, synced)}
      </StatusPill>
      <StatusPill tone="neutral">
        {blockchain.data?.chain === 'test' ? 'Testnet' : 'Mainnet'}
      </StatusPill>
    </>
  );
}

function buildWalletSection(coin: CoinId, data: DashboardData): ReactNode {
  const wallet = data.effectiveWallet;

  return (
    <HeroSection title="Wallet" icon={<Wallet />}>
      <MiniStat
        label="Balance"
        value={
          wallet ? (
            <span className={lockedWalletBalanceClass(wallet)}>
              {formatCoinAmount(wallet.balance, coin, 4)}
            </span>
          ) : (
            '—'
          )
        }
      />
      <MiniStat
        label="Immature"
        value={
          wallet ? (
            <span className={lockedWalletBalanceClass(wallet)}>
              {formatCoinAmount(wallet.immature_balance, coin, 4)}
            </span>
          ) : (
            '—'
          )
        }
      />
      {coin === 'vericoin' ? (
        <MiniStat
          label="Stake weight"
          value={
            wallet && data.vrcMining.data?.stakeweight?.combined != null ? (
              <span className={lockedWalletBalanceClass(wallet)}>
                {formatNumber(data.vrcMining.data.stakeweight.combined, 0)}
              </span>
            ) : (
              '—'
            )
          }
        />
      ) : (
        <MiniStat
          label="Unconfirmed"
          value={
            wallet ? (
              <span className={lockedWalletBalanceClass(wallet)}>
                {formatCoinAmount(wallet.unconfirmed_balance, coin, 4)}
              </span>
            ) : (
              '—'
            )
          }
        />
      )}
      <MiniStat
        label="Transactions"
        value={
          wallet ? (
            <span className={lockedWalletBalanceClass(wallet)}>
              {formatNumber(wallet.txcount, 0)}
            </span>
          ) : (
            '—'
          )
        }
      />
    </HeroSection>
  );
}

function buildMarketSection(coin: CoinId, profile: CoinProfile, data: DashboardData): ReactNode {
  const stats = data.explorer.data;
  const vrcNetwork =
    coin === 'vericoin' ? mergeStakingNetworkKpis(data.vrcMining.data, stats) : null;

  return (
    <HeroSection
      title="Market"
      icon={<TrendingUp />}
      action={<ExplorerLink coin={coin} target={{ kind: 'home' }} label="Explorer" />}
    >
      <MiniStat label={profile.symbol} value={formatUsd(stats?.price_usd)} />
      <MiniStat label="24h vol" value={formatUsd(stats?.volume_24h_usd)} />
      <MiniStat
        label={coin === 'vericoin' ? 'Interest rate' : 'Block reward'}
        value={
          coin === 'vericoin'
            ? vrcNetwork?.interestRate != null
              ? `${formatNumber(vrcNetwork.interestRate, 2)}%`
              : stats?.stake_interest != null
                ? `${formatNumber(stats.stake_interest, 2)}%`
                : '—'
            : stats?.block_reward != null
              ? `${formatNumber(stats.block_reward, 4)} ${profile.symbol}`
              : '—'
        }
      />
      <MiniStat
        label="Supply"
        value={stats?.supply != null ? formatNumber(stats.supply, 0) : '—'}
      />
    </HeroSection>
  );
}

function buildActivitySection(
  coin: CoinId,
  data: DashboardData,
  sectionClassName?: string
): ReactNode {
  if (data.isLight) {
    return (
      <HeroSection title="Connection" icon={<Users />} className={sectionClassName}>
        <MiniStat label="Server" value={data.connected ? 'Online' : 'Offline'} />
        <MiniStat
          label="Chain tip"
          value={data.tipHeight != null ? formatNumber(data.tipHeight, 0) : '—'}
        />
        <MiniStat
          label="Sync"
          value={data.wallet.data?.light_syncing ? 'Syncing' : data.connected ? 'Ready' : '—'}
        />
        <MiniStat label="Mode" value="Light wallet" />
      </HeroSection>
    );
  }

  if (coin === 'verium') {
    const minerBooting = data.poolMinerRunning
      ? data.localHashrate <= 0
      : isMinerBooting(data.minerActive, data.localHashrate, data.minerState.data?.started_at);
    const networkStats = buildNetworkStats(data.explorer.data, data.mining.data);
    const share = networkSharePercent(data.localHashrate, networkStats?.networkHash);
    const blocksFound =
      data.transactions.data?.filter((t) => t.category === 'generate' || t.category === 'immature')
        .length ?? 0;
    const daily =
      networkStats && data.localHashrate > 0
        ? estimateDailyMining({
            localHashrateHm: data.localHashrate,
            networkHashrateHs: networkStats.networkHash!,
            blocksPerHour: networkStats.blocksPerHour!,
            blockReward: networkStats.blockReward!,
            priceUsd: networkStats.priceUsd,
          })
        : null;

    return (
      <HeroSection
        title="Your mining"
        icon={
          <MiningPickaxeAnimation
            active={data.miningActive && !minerBooting}
            booting={minerBooting}
          />
        }
        action={<MinerBootBadge booting={minerBooting} active={data.miningActive} />}
        href="/mining"
        className={sectionClassName}
      >
        <MiniStat label="Blocks found" value={formatNumber(blocksFound, 0)} />
        <MiniStat
          label="Hashrate"
          value={
            <AnimatedHashrate
              booting={minerBooting}
              value={data.localHashrate > 0 ? data.localHashrate : undefined}
              fractionDigits={0}
              className="font-semibold text-fg"
              immediate={data.miningActive}
            />
          }
        />
        <MiniStat
          label="Network share"
          value={share != null ? `${formatNumber(share, 2)}%` : '—'}
        />
        <MiniStat
          label="Est. daily"
          value={daily ? `${formatNumber(daily.vrmPerDay, 3)} VRM` : '—'}
        />
      </HeroSection>
    );
  }

  const vrcNetwork = mergeStakingNetworkKpis(data.vrcMining.data, data.explorer.data);
  const vrcNetworkStakePct = networkCoinsStakingPercent(vrcNetwork.netStakeWeight);
  const vrcStakeShare = walletStakeSharePercent(data.wallet.data?.stake, vrcNetwork.netStakeWeight);
  const stakingActive = data.stakingState.data?.active ?? false;
  const stakeTxCount =
    data.transactions.data?.filter(
      (t) => t.category === 'stake' || t.category === 'stake-mint' || t.category === 'stake-orphan'
    ).length ?? 0;

  return (
    <HeroSection
      title="Your staking"
      icon={<Coins className={cn('h-4 w-4', stakingActive ? 'text-success' : 'text-accent')} />}
      action={
        <span
          className={cn(
            'rounded-full px-2 py-0.5 text-[10px] font-semibold',
            stakingActive ? 'bg-success/12 text-success' : 'bg-bg-subtle text-fg-muted'
          )}
        >
          {stakingActive ? 'Active' : 'Inactive'}
        </span>
      }
      href="/staking"
      className={sectionClassName}
    >
      <MiniStat label="Stake rewards" value={formatNumber(stakeTxCount, 0)} />
      <MiniStat
        label="Interest rate"
        value={
          vrcNetwork.interestRate != null ? `${formatNumber(vrcNetwork.interestRate, 2)}%` : '—'
        }
      />
      <MiniStat
        label="Network staked"
        value={vrcNetworkStakePct != null ? `${formatNumber(vrcNetworkStakePct, 2)}%` : '—'}
      />
      <MiniStat
        label="Stake share"
        value={vrcStakeShare != null ? `${formatNumber(vrcStakeShare, 2)}%` : '—'}
      />
    </HeroSection>
  );
}

function buildDetailColumns(
  coin: CoinId,
  profile: CoinProfile,
  data: DashboardData,
  explorerEnabled: boolean
): ReactNode {
  const activitySpan = explorerEnabled ? 'lg:col-span-2 xl:col-span-1' : undefined;

  return (
    <>
      {buildWalletSection(coin, data)}
      {explorerEnabled ? buildMarketSection(coin, profile, data) : null}
      {buildActivitySection(coin, data, activitySpan)}
    </>
  );
}

function buildNetworkMetrics(coin: CoinId, data: DashboardData): ReactNode {
  const peerLabel =
    data.connections > 0 ? formatNumber(data.connections, 0) : data.connected ? '0' : '—';
  const peerSub = data.connections > 0 ? 'Online' : data.connected ? 'No peers' : 'Offline';

  if (coin === 'verium') {
    const networkHashKhm =
      data.explorer.data?.network_hash != null
        ? networkHashToKhm(data.explorer.data.network_hash)
        : data.mining.data?.networkhashps != null
          ? networkHashToKhm(data.mining.data.networkhashps)
          : null;
    const difficultyValue =
      data.explorer.data?.difficulty ??
      data.blockchain.data?.difficulty ??
      data.mining.data?.difficulty;
    const blockTimeMin = resolveBlockTimeMinutes(data.explorer.data, data.mining.data);
    const mempool = data.mining.data?.pooledtx ?? data.explorer.data?.pooled_tx;

    return (
      <>
        <NetworkMetric
          label="Network hashrate"
          value={networkHashKhm != null ? `${formatNumber(networkHashKhm, 1)} kH/m` : '—'}
        />
        <NetworkMetric
          label="Difficulty"
          value={
            difficultyValue != null
              ? difficultyValue >= 0.0001
                ? formatNumber(difficultyValue, 4)
                : formatNumber(difficultyValue, 6)
              : '—'
          }
        />
        <NetworkMetric
          label="Avg. block time"
          value={blockTimeMin != null ? `${formatNumber(blockTimeMin, 1)} min` : '—'}
        />
        <NetworkMetric label="Mempool" value={mempool != null ? formatNumber(mempool, 0) : '—'} />
        <NetworkMetric label={`Peers · ${peerSub}`} value={peerLabel} />
      </>
    );
  }

  const vrcNetwork = mergeStakingNetworkKpis(data.vrcMining.data, data.explorer.data);
  const networkStakePct = networkCoinsStakingPercent(vrcNetwork.netStakeWeight);
  const mempool = data.vrcMining.data?.pooledtx ?? data.explorer.data?.pooled_tx;
  const posDifficulty = vrcNetwork.posDifficulty ?? data.blockchain.data?.difficulty;
  const blockTimeMin = resolveBlockTimeMinutes(data.explorer.data, null);

  return (
    <>
      <NetworkMetric
        label="PoS difficulty"
        value={
          posDifficulty != null
            ? posDifficulty >= 0.0001
              ? formatNumber(posDifficulty, 4)
              : formatNumber(posDifficulty, 6)
            : '—'
        }
      />
      <NetworkMetric
        label="Network staked"
        value={networkStakePct != null ? `${formatNumber(networkStakePct, 2)}%` : '—'}
      />
      <NetworkMetric
        label="Block time"
        value={blockTimeMin != null ? `${formatNumber(blockTimeMin, 1)} min` : '—'}
      />
      <NetworkMetric label="Mempool" value={mempool != null ? formatNumber(mempool, 0) : '—'} />
      <NetworkMetric label={`Peers · ${peerSub}`} value={peerLabel} />
    </>
  );
}

export function DashboardHero({ coin }: { coin: CoinId }) {
  const profile = getCoinProfile(coin);
  const explorerEnabled = useExplorerQueriesEnabled();
  const data = useDashboardData(coin);

  return (
    <HeroPanel
      profile={profile}
      statusRow={buildHeroStatusRow(data)}
      localBlocks={data.localBlocks}
      activity={data.activity}
      synced={data.synced}
      syncTarget={data.syncTarget}
      behind={data.behind}
      detailColumns={buildDetailColumns(coin, profile, data, explorerEnabled)}
      networkMetrics={buildNetworkMetrics(coin, data)}
    />
  );
}
