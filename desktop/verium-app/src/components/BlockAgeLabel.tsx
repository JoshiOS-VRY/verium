import { Clock3 } from 'lucide-react';

import { useBlockAgeTick } from '@/hooks/useBlockAgeTick';
import { formatBlockAge } from '@/lib/utils';

/**
 * Renders the "Mined <age> ago" line for the dashboard hero. The ~1Hz tick that
 * keeps the age fresh lives here (not in `useDashboardData`) so only this tiny
 * leaf re-renders each second instead of the entire hero query bundle.
 */
export function BlockAgeLabel({
  tipTime,
  enabled = true,
}: {
  tipTime: number | null | undefined;
  enabled?: boolean;
}) {
  const tick = useBlockAgeTick(enabled && tipTime != null);
  const blockAge = tipTime != null ? formatBlockAge(tipTime, tick) : '—';
  if (blockAge === '—') return null;

  return (
    <p className="mt-3 inline-flex min-w-0 items-center gap-1.5 text-sm text-fg-muted xl:mt-2">
      <Clock3 className="h-3.5 w-3.5 shrink-0 opacity-70" aria-hidden />
      <span>
        Mined <span className="font-medium text-fg">{blockAge}</span> ago
      </span>
    </p>
  );
}
