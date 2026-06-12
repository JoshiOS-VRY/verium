import type { ReactNode } from 'react';
import { Box, Calendar, Hash, Layers, Pickaxe } from 'lucide-react';
import { formatBlockDifficulty } from '@/components/ExplorerRecentBlockRow';
import type { IndexerBlockSummary } from '@/lib/indexer-api';
import { formatTransactionTime } from '@/lib/units';

function StatCard({
  icon,
  label,
  value,
  sub,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
  sub?: ReactNode;
}) {
  return (
    <div className="min-w-[9.5rem] shrink-0 rounded-xl border border-border/70 bg-bg-subtle/30 px-3 py-2.5">
      <div className="flex items-center gap-1.5 text-fg-muted">
        {icon}
        <span className="text-[10px] font-medium uppercase tracking-wide">{label}</span>
      </div>
      <p className="mt-1 text-sm font-semibold tabular-nums text-fg">{value}</p>
      {sub ? <p className="mt-0.5 text-[10px] text-fg-subtle">{sub}</p> : null}
    </div>
  );
}

function formatBlockSize(bytes?: number): { value: string; sub?: string } {
  if (bytes == null) return { value: '—' };
  if (bytes < 1024) return { value: `${bytes.toLocaleString()} B` };
  const kb = bytes / 1024;
  if (kb < 1024) return { value: `${kb.toFixed(1)} KB`, sub: `${bytes.toLocaleString()} bytes` };
  return { value: `${(kb / 1024).toFixed(2)} MB`, sub: `${bytes.toLocaleString()} bytes` };
}

export function BlockStatsStrip({ block }: { block: IndexerBlockSummary }) {
  const txCount = block.txCount != null ? block.txCount.toLocaleString() : '—';
  const size = formatBlockSize(block.size);
  const difficulty = formatBlockDifficulty(block.difficulty ?? undefined);

  return (
    <section className="-mx-1 flex gap-2 overflow-x-auto px-1 pb-1 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
      <StatCard
        icon={<Hash className="h-3.5 w-3.5" />}
        label="Transactions"
        value={txCount}
        sub="In this block"
      />
      <StatCard
        icon={<Box className="h-3.5 w-3.5" />}
        label="Size"
        value={size.value}
        sub={size.sub}
      />
      <StatCard
        icon={<Pickaxe className="h-3.5 w-3.5 text-amber-600 dark:text-amber-400" />}
        label="Difficulty"
        value={difficulty}
      />
      {block.time != null && (
        <StatCard
          icon={<Calendar className="h-3.5 w-3.5" />}
          label="Timestamp"
          value={formatTransactionTime(block.time)}
        />
      )}
      <StatCard
        icon={<Layers className="h-3.5 w-3.5" />}
        label="Height"
        value={`#${block.height.toLocaleString()}`}
      />
    </section>
  );
}
