/** iPad/tablet: both dimensions ≥768px (excludes phone portrait and landscape). */
export const MOBILE_TABLET_MEDIA = '(min-width: 768px) and (min-height: 768px)';

/** Large iPad / landscape: wider layouts and sidebar labels. */
export const MOBILE_LARGE_TABLET_MEDIA = '(min-width: 1024px) and (min-height: 768px)';

export function isMobileTabletViewport(): boolean {
  if (typeof window === 'undefined' || !window.matchMedia) return false;
  return window.matchMedia(MOBILE_TABLET_MEDIA).matches;
}

export function isMobileLargeTabletViewport(): boolean {
  if (typeof window === 'undefined' || !window.matchMedia) return false;
  return window.matchMedia(MOBILE_LARGE_TABLET_MEDIA).matches;
}
