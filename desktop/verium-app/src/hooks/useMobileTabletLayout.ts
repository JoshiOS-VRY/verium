import { useEffect, useState } from 'react';
import {
  isMobileLargeTabletViewport,
  isMobileTabletViewport,
  MOBILE_LARGE_TABLET_MEDIA,
  MOBILE_TABLET_MEDIA,
} from '@/lib/mobile-viewport';

/** True on iPad-sized viewports inside the mobile shell (not iPhone). */
export function useMobileTabletLayout(): boolean {
  const [isTablet, setIsTablet] = useState(isMobileTabletViewport);

  useEffect(() => {
    if (typeof window === 'undefined' || !window.matchMedia) return;
    const media = window.matchMedia(MOBILE_TABLET_MEDIA);
    const sync = () => setIsTablet(media.matches);
    sync();
    media.addEventListener('change', sync);
    return () => media.removeEventListener('change', sync);
  }, []);

  return isTablet;
}

/** True on large iPad / landscape tablet widths. */
export function useMobileLargeTabletLayout(): boolean {
  const [isLargeTablet, setIsLargeTablet] = useState(isMobileLargeTabletViewport);

  useEffect(() => {
    if (typeof window === 'undefined' || !window.matchMedia) return;
    const media = window.matchMedia(MOBILE_LARGE_TABLET_MEDIA);
    const sync = () => setIsLargeTablet(media.matches);
    sync();
    media.addEventListener('change', sync);
    return () => media.removeEventListener('change', sync);
  }, []);

  return isLargeTablet;
}
