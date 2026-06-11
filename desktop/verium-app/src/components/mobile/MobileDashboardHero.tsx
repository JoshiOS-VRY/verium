import { ChevronDown, TrendingUp } from "lucide-react";
import { AnimatedBlockNumber } from "@/components/AnimatedBlockNumber";
import { BlockAgeLabel } from "@/components/BlockAgeLabel";
import { ExplorerLink } from "@/components/ExplorerLink";
import { useDashboardData } from "@/hooks/useDashboardData";
import { getCoinProfile, type CoinId } from "@/lib/coin/profile";
import { formatCoinAmount } from "@/lib/units";
import { cn, formatNumber } from "@/lib/utils";
import { lockedWalletBalanceClass } from "@/lib/wallet-unlock";

function formatUsd(value?: number | null): string {
  if (value === undefined || value === null) return "—";
  if (value >= 1_000_000) return `$${formatNumber(value / 1_000_000, 2)}M`;
  if (value >= 1_000) return `$${formatNumber(value / 1_000, 2)}K`;
  return `$${formatNumber(value, 4)}`;
}

export function MobileDashboardHero({ coin }: { coin: CoinId }) {
  const profile = getCoinProfile(coin);
  const data = useDashboardData(coin);
  const wallet = data.effectiveWallet;
  const priceUsd = data.explorer.data?.price_usd;
  const online = data.connected;

  return (
    <section className="mobile-panel overflow-hidden rounded-2xl border border-border bg-bg-panel shadow-sm">
      <div className="flex items-center justify-between gap-2 border-b border-border/60 px-4 py-3">
        <span
          className={cn(
            "inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-[11px] font-semibold",
            online
              ? "bg-success/10 text-success"
              : "bg-warning/10 text-warning",
          )}
        >
          <span
            className={cn(
              "h-1.5 w-1.5 rounded-full",
              online ? "bg-success" : "bg-warning",
            )}
          />
          {online ? "Connected" : "Offline"}
        </span>
        <span className="text-[11px] font-medium text-fg-subtle">Light wallet</span>
      </div>

      <div className="px-4 py-4">
        <div className="flex items-end justify-between gap-3">
          <div className="min-w-0">
            <p className="text-[11px] font-medium uppercase tracking-wide text-fg-subtle">
              Chain tip
            </p>
            <div className="mt-1">
              <AnimatedBlockNumber
                value={data.tipHeight}
                className="text-2xl font-bold tabular-nums text-fg"
                fallback={data.activity.showSpinner ? "…" : "—"}
              />
            </div>
            <div className="mt-1 [&_p]:mt-0 [&_p]:text-xs">
              <BlockAgeLabel tipTime={data.tipTime} />
            </div>
          </div>
          {data.tipHash && (
            <ExplorerLink
              coin={coin}
              target={{ kind: "block", hashOrHeight: data.tipHash }}
              label="View"
              className="shrink-0 text-xs"
            />
          )}
        </div>

        {priceUsd != null && (
          <div className="mt-4 flex items-center gap-2 rounded-xl bg-bg-subtle/50 px-3 py-2.5">
            <TrendingUp className="h-4 w-4 shrink-0 text-accent" />
            <div className="min-w-0">
              <p className="text-[10px] font-medium uppercase tracking-wide text-fg-subtle">
                {profile.symbol} price
              </p>
              <p className="text-sm font-semibold tabular-nums text-fg">
                {formatUsd(priceUsd)}
              </p>
            </div>
          </div>
        )}
      </div>

      <details className="group border-t border-border/60">
        <summary className="flex cursor-pointer list-none items-center justify-between px-4 py-3 text-xs font-medium text-fg-muted marker:content-none">
          <span>Network details</span>
          <ChevronDown className="h-4 w-4 transition-transform group-open:rotate-180" />
        </summary>
        <dl className="grid grid-cols-2 gap-3 px-4 pb-4 text-xs">
          <div>
            <dt className="text-fg-subtle">Wallet balance</dt>
            <dd
              className={cn(
                "mt-0.5 font-semibold tabular-nums text-fg",
                wallet && lockedWalletBalanceClass(wallet),
              )}
            >
              {wallet
                ? formatCoinAmount(wallet.balance, coin, 4)
                : "—"}
            </dd>
          </div>
          <div>
            <dt className="text-fg-subtle">Mempool</dt>
            <dd className="mt-0.5 font-semibold tabular-nums text-fg">
              {data.explorer.data?.pooled_tx != null
                ? formatNumber(data.explorer.data.pooled_tx, 0)
                : "—"}
            </dd>
          </div>
          <div className="col-span-2">
            <dt className="text-fg-subtle">Status</dt>
            <dd className="mt-0.5 text-fg">{data.activity.title}</dd>
          </div>
        </dl>
      </details>
    </section>
  );
}
