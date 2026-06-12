import type { NavigateFunction } from 'react-router-dom';
import { isExplorerDetailPath } from '@/lib/explorer-nav';

/** Bottom-tab routes: back opens the wallet menu hub. */
export const MOBILE_TAB_ROOT_PATHS = [
  '/dashboard',
  '/transactions',
  '/addresses',
  '/settings',
] as const;

export function isMobileTabRoot(pathname: string): boolean {
  return (MOBILE_TAB_ROOT_PATHS as readonly string[]).includes(pathname);
}

/** Whether edge-swipe back is meaningful on this route. */
export function canMobileEdgeSwipeBack(pathname: string): boolean {
  return pathname !== '/setup';
}

function hasBrowserHistory(): boolean {
  return typeof window !== 'undefined' && window.history.length > 1;
}

/** Match the header back button: tab roots → wallet menu; else history with safe fallbacks. */
export function performMobileBack(navigate: NavigateFunction, pathname: string): void {
  if (pathname === '/setup') return;

  if (isMobileTabRoot(pathname)) {
    navigate('/setup', { state: { setupHub: true } });
    return;
  }

  if (hasBrowserHistory()) {
    navigate(-1);
    return;
  }

  if (isExplorerDetailPath(pathname)) {
    navigate('/transactions');
    return;
  }

  navigate('/dashboard');
}

export function mobileBackAriaLabel(pathname: string): string {
  if (isMobileTabRoot(pathname)) return 'Back to wallet menu';
  if (isExplorerDetailPath(pathname)) return 'Back';
  return 'Back';
}
