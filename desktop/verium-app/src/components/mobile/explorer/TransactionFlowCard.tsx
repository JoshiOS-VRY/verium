import { ArrowDown, ArrowRight, Coins, Hammer } from 'lucide-react';
import type { IndexerVin, IndexerVout } from '@/lib/indexer-api';
import {
  formatIndexerCoinsTotal,
  indexerTicker,
  parseIndexerAmountCoins,
  sumIndexerAmountCoins,
} from '@/lib/indexer-amount';
import { cn } from '@/lib/utils';

const IO_PREVIEW_LIMIT = 6;

function FlowPill({
  label,
  total,
  ticker,
  count,
  tone,
}: {
  label: string;
  total: number;
  ticker: string;
  count: number;
  tone: 'input' | 'output';
}) {
  return (
    <div
      className={cn(
        'min-w-0 flex-1 rounded-xl border px-3 py-2.5',
        tone === 'input'
          ? 'border-amber-500/25 bg-amber-500/8'
          : 'border-emerald-500/25 bg-emerald-500/8'
      )}
    >
      <p
        className={cn(
          'text-[11px] font-medium uppercase tracking-wide',
          tone === 'input' ? 'text-amber-700 dark:text-amber-300' : 'text-emerald-700 dark:text-emerald-300'
        )}
      >
        {label}
      </p>
      <p className="mt-1 text-base font-semibold tabular-nums text-fg">
        {formatIndexerCoinsTotal(total, ticker)}
      </p>
      <p className="mt-0.5 text-[11px] text-fg-muted">
        {count === 1 ? '1 leg' : `${count} legs`}
      </p>
    </div>
  );
}

export function TransactionFlowCard({
  inputs,
  outputs,
  isCoinbase,
  isCoinstake,
}: {
  inputs: IndexerVin[];
  outputs: IndexerVout[];
  isCoinbase?: boolean;
  isCoinstake?: boolean;
}) {
  const inputTotal = sumIndexerAmountCoins(inputs.map((v) => v.value));
  const outputTotal = sumIndexerAmountCoins(outputs.map((v) => v.value));
  const ticker =
    indexerTicker(inputs.find((v) => v.value)?.value) ||
    indexerTicker(outputs.find((v) => v.value)?.value) ||
    'VRM';

  const fee = inputTotal > 0 && outputTotal > 0 ? inputTotal - outputTotal : 0;
  const showFee = fee > 0.00000001;

  const inputPreview = inputs.slice(0, IO_PREVIEW_LIMIT);
  const outputPreview = outputs.slice(0, IO_PREVIEW_LIMIT);
  const hiddenInputs = inputs.length - inputPreview.length;
  const hiddenOutputs = outputs.length - outputPreview.length;

  return (
    <section className="mobile-panel rounded-2xl border border-border bg-bg-panel/60 p-4">
      <div className="flex items-center gap-2">
        <Coins className="h-4 w-4 text-accent" aria-hidden />
        <h3 className="text-sm font-semibold text-fg">Money flow</h3>
        {(isCoinbase || isCoinstake) && (
          <span className="inline-flex items-center gap-1 rounded-full border border-border bg-bg-subtle px-2 py-0.5 text-[10px] font-medium text-fg-muted">
            {isCoinbase ? (
              <>
                <Hammer className="h-3 w-3" aria-hidden />
                Coinbase
              </>
            ) : (
              'Coinstake'
            )}
          </span>
        )}
      </div>

      {isCoinbase && inputs.length <= 1 ? (
        <div className="mt-3 rounded-xl border border-dashed border-border/80 bg-bg-subtle/40 px-3 py-3 text-xs text-fg-muted">
          New coins minted in this block — no prior inputs.
        </div>
      ) : null}

      <div className="mt-3 flex items-stretch gap-2">
        <FlowPill
          label="Inputs"
          total={inputTotal}
          ticker={ticker}
          count={inputs.length}
          tone="input"
        />
        <div className="flex shrink-0 flex-col items-center justify-center px-0.5 text-fg-muted">
          <ArrowRight className="h-4 w-4 hidden min-[380px]:block" aria-hidden />
          <ArrowDown className="h-4 w-4 min-[380px]:hidden" aria-hidden />
        </div>
        <FlowPill
          label="Outputs"
          total={outputTotal}
          ticker={ticker}
          count={outputs.length}
          tone="output"
        />
      </div>

      {showFee && (
        <p className="mt-2 text-center text-[11px] text-fg-muted">
          Network fee{' '}
          <span className="font-medium tabular-nums text-fg">
            {formatIndexerCoinsTotal(fee, ticker)}
          </span>
        </p>
      )}

      {(inputPreview.length > 0 || outputPreview.length > 0) && (
        <div className="mt-4 space-y-3 border-t border-border/60 pt-3">
          {inputPreview.length > 0 && (
            <div>
              <p className="text-[11px] font-medium uppercase tracking-wide text-fg-muted">
                Input preview
              </p>
              <ul className="mt-1.5 space-y-1">
                {inputPreview.map((vin) => (
                  <li
                    key={vin.n}
                    className="flex items-center justify-between gap-2 text-[11px] tabular-nums"
                  >
                    <span className="truncate text-fg-muted">#{vin.n}</span>
                    <span className="shrink-0 font-medium text-fg">
                      {formatIndexerCoinsTotal(parseIndexerAmountCoins(vin.value), ticker)}
                    </span>
                  </li>
                ))}
                {hiddenInputs > 0 && (
                  <li className="text-[11px] text-fg-subtle">+{hiddenInputs} more inputs below</li>
                )}
              </ul>
            </div>
          )}
          {outputPreview.length > 0 && (
            <div>
              <p className="text-[11px] font-medium uppercase tracking-wide text-fg-muted">
                Output preview
              </p>
              <ul className="mt-1.5 space-y-1">
                {outputPreview.map((vout) => (
                  <li
                    key={vout.n}
                    className="flex items-center justify-between gap-2 text-[11px] tabular-nums"
                  >
                    <span className="truncate text-fg-muted">#{vout.n}</span>
                    <span className="shrink-0 font-medium text-fg">
                      {formatIndexerCoinsTotal(parseIndexerAmountCoins(vout.value), ticker)}
                    </span>
                  </li>
                ))}
                {hiddenOutputs > 0 && (
                  <li className="text-[11px] text-fg-subtle">+{hiddenOutputs} more outputs below</li>
                )}
              </ul>
            </div>
          )}
        </div>
      )}
    </section>
  );
}
