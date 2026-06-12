import { useCallback, useEffect, useRef, useState, type CSSProperties } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { canMobileEdgeSwipeBack, performMobileBack } from '@/lib/mobile-back-nav';

const EDGE_ZONE_PX = 22;
const MAX_VERTICAL_DRIFT_PX = 48;
const COMPLETE_RATIO = 0.34;
const MIN_FLING_VELOCITY = 0.45; // px/ms
const ANIM_MS = 320;
const IOS_EASE = 'cubic-bezier(0.32, 0.72, 0, 1)';

function rubberBand(value: number, limit: number): number {
  if (value <= limit) return value;
  const excess = value - limit;
  return limit + excess * 0.22;
}

/**
 * iOS-style interactive edge swipe: content follows the finger, scrim + shadow, then pops back.
 */
export function useMobileInteractiveBack(
  containerRef: React.RefObject<HTMLElement | null>,
  enabled = true
) {
  const navigate = useNavigate();
  const { pathname } = useLocation();
  const [dragX, setDragX] = useState(0);
  const [transitioning, setTransitioning] = useState(false);
  const tracking = useRef(false);
  const start = useRef({ x: 0, y: 0, t: 0 });
  const dragXRef = useRef(0);
  const pathnameRef = useRef(pathname);

  pathnameRef.current = pathname;
  dragXRef.current = dragX;

  const canBack = enabled && canMobileEdgeSwipeBack(pathname);
  const screenWidth = typeof window !== 'undefined' ? window.innerWidth : 390;
  const completeThreshold = screenWidth * COMPLETE_RATIO;

  const resetGesture = useCallback(() => {
    tracking.current = false;
    setTransitioning(true);
    setDragX(0);
    window.setTimeout(() => setTransitioning(false), ANIM_MS);
  }, []);

  const finishBack = useCallback(() => {
    tracking.current = false;
    setTransitioning(true);
    setDragX(screenWidth);
    window.setTimeout(() => {
      performMobileBack(navigate, pathnameRef.current);
      setDragX(0);
      setTransitioning(false);
    }, ANIM_MS);
  }, [navigate, screenWidth]);

  useEffect(() => {
    const el = containerRef.current;
    if (!el || !canBack) return;

    const resetTracking = () => {
      tracking.current = false;
    };

    const onTouchStart = (e: TouchEvent) => {
      if (transitioning || e.touches.length !== 1) return;
      const touch = e.touches[0];
      if (touch.clientX > EDGE_ZONE_PX) return;
      tracking.current = true;
      start.current = { x: touch.clientX, y: touch.clientY, t: Date.now() };
    };

    const onTouchMove = (e: TouchEvent) => {
      if (!tracking.current || e.touches.length !== 1) return;
      const touch = e.touches[0];
      const dx = touch.clientX - start.current.x;
      const dy = touch.clientY - start.current.y;

      if (dx < 0) {
        setDragX(0);
        return;
      }

      if (Math.abs(dy) > MAX_VERTICAL_DRIFT_PX && Math.abs(dy) > dx * 0.85) {
        resetTracking();
        resetGesture();
        return;
      }

      if (dx > 8) {
        e.preventDefault();
      }

      setTransitioning(false);
      setDragX(rubberBand(dx, screenWidth));
    };

    const onTouchEnd = () => {
      if (!tracking.current) return;
      const dx = dragXRef.current;
      const dt = Math.max(Date.now() - start.current.t, 1);
      const velocity = dx / dt;

      if (dx >= completeThreshold || velocity >= MIN_FLING_VELOCITY) {
        finishBack();
      } else {
        resetGesture();
      }
      resetTracking();
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
  }, [
    containerRef,
    canBack,
    completeThreshold,
    finishBack,
    resetGesture,
    screenWidth,
    transitioning,
  ]);

  const progress = Math.min(dragX / completeThreshold, 1);
  const scrimOpacity = progress * 0.38;
  const shadowStrength = Math.min(progress, 1);

  const contentStyle: CSSProperties = {
    transform: dragX > 0 ? `translate3d(${dragX}px, 0, 0)` : undefined,
    transition: transitioning ? `transform ${ANIM_MS}ms ${IOS_EASE}` : undefined,
    willChange: dragX > 0 || transitioning ? 'transform' : undefined,
    boxShadow:
      dragX > 0 ? `-10px 0 28px rgba(0, 0, 0, ${0.12 + shadowStrength * 0.18})` : undefined,
  };

  const scrimStyle: CSSProperties = {
    opacity: scrimOpacity,
    transition: transitioning ? `opacity ${ANIM_MS}ms ${IOS_EASE}` : undefined,
    pointerEvents: 'none',
  };

  return {
    canBack,
    contentStyle,
    scrimStyle,
    isDragging: dragX > 0 && !transitioning,
  };
}
