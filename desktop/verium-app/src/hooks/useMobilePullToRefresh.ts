import { useEffect, useRef, useState } from 'react';
import { useMobileScrollContainer } from '@/contexts/MobileScrollContext';

const PULL_THRESHOLD = 72;
const MAX_PULL = 110;

export function useMobilePullToRefresh(onRefresh: () => Promise<void>, enabled = true) {
  const scrollRef = useMobileScrollContainer();
  const [pullDistance, setPullDistance] = useState(0);
  const [refreshing, setRefreshing] = useState(false);
  const startY = useRef(0);
  const pulling = useRef(false);
  const canPull = useRef(false);
  const pullDistanceRef = useRef(0);
  const onRefreshRef = useRef(onRefresh);

  onRefreshRef.current = onRefresh;

  useEffect(() => {
    pullDistanceRef.current = pullDistance;
  }, [pullDistance]);

  useEffect(() => {
    const el = scrollRef?.current;
    if (!el || !enabled) return;

    const resetPull = () => {
      pulling.current = false;
      canPull.current = false;
      setPullDistance(0);
    };

    const onTouchStart = (e: TouchEvent) => {
      if (refreshing) return;
      if (el.scrollTop <= 0) {
        canPull.current = true;
        startY.current = e.touches[0].clientY;
      } else {
        canPull.current = false;
      }
    };

    const onTouchMove = (e: TouchEvent) => {
      if (!canPull.current || refreshing) return;
      const y = e.touches[0].clientY;
      const delta = y - startY.current;
      if (delta > 0 && el.scrollTop <= 0) {
        e.preventDefault();
        pulling.current = true;
        const distance = Math.min(delta * 0.45, MAX_PULL);
        pullDistanceRef.current = distance;
        setPullDistance(distance);
      } else if (delta <= 0) {
        pulling.current = false;
        pullDistanceRef.current = 0;
        setPullDistance(0);
      }
    };

    const onTouchEnd = async () => {
      const distance = pullDistanceRef.current;
      if (!pulling.current && distance === 0) return;

      if (distance >= PULL_THRESHOLD && !refreshing) {
        setRefreshing(true);
        setPullDistance(PULL_THRESHOLD);
        try {
          await onRefreshRef.current();
        } finally {
          setRefreshing(false);
          resetPull();
        }
      } else {
        resetPull();
      }
    };

    el.addEventListener('touchstart', onTouchStart, { passive: true });
    el.addEventListener('touchmove', onTouchMove, { passive: false });
    el.addEventListener('touchend', onTouchEnd);
    el.addEventListener('touchcancel', onTouchEnd);

    return () => {
      el.removeEventListener('touchstart', onTouchStart);
      el.removeEventListener('touchmove', onTouchMove);
      el.removeEventListener('touchend', onTouchEnd);
      el.removeEventListener('touchcancel', onTouchEnd);
    };
  }, [scrollRef, enabled, refreshing]);

  return {
    pullDistance,
    refreshing,
    threshold: PULL_THRESHOLD,
    active: pullDistance > 0 || refreshing,
  };
}
