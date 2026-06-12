import { Loader2 } from 'lucide-react';
import { useLayoutEffect, useState } from 'react';
import { createPortal } from 'react-dom';
import { useMobileScrollContainer } from '@/contexts/MobileScrollContext';
import { useMobilePullToRefresh } from '@/hooks/useMobilePullToRefresh';
import { cn } from '@/lib/utils';

/** iOS-style pull-down refresh indicator; touch handlers attach to the mobile scroll container. */
export function MobilePullToRefresh({
  onRefresh,
  enabled = true,
}: {
  onRefresh: () => Promise<void>;
  enabled?: boolean;
}) {
  const scrollRef = useMobileScrollContainer();
  const [scrollReady, setScrollReady] = useState(false);
  const { pullDistance, refreshing, threshold, active } = useMobilePullToRefresh(
    onRefresh,
    enabled
  );

  useLayoutEffect(() => {
    if (scrollRef?.current) {
      setScrollReady(true);
    }
  }, [scrollRef]);

  const scrollEl = scrollRef?.current;
  if (!scrollReady || !scrollEl || (!enabled && !active)) return null;

  const progress = Math.min(pullDistance / threshold, 1);
  const ready = pullDistance >= threshold;

  const indicator = (
    <div
      className="mobile-pull-refresh pointer-events-none absolute inset-x-0 top-0 z-20 flex justify-center"
      style={{
        height: Math.max(pullDistance, refreshing ? threshold : 0),
        transition: active && !refreshing ? 'none' : 'height 0.2s ease-out',
      }}
    >
      <div
        className={cn(
          'mt-2 flex h-8 w-8 items-center justify-center rounded-full border border-border/80 bg-bg-panel shadow-sm',
          ready || refreshing ? 'text-accent' : 'text-fg-muted'
        )}
        style={{
          opacity: active ? Math.max(progress, refreshing ? 1 : 0.35) : 0,
          transform: `scale(${0.85 + progress * 0.15}) rotate(${progress * 180}deg)`,
          transition: active && !refreshing ? 'none' : 'opacity 0.2s ease-out',
        }}
      >
        <Loader2 className={cn('h-4 w-4', refreshing && 'animate-spin')} aria-hidden />
      </div>
    </div>
  );

  return createPortal(indicator, scrollEl);
}
