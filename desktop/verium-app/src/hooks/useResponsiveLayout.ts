import { useWalletMode } from '@/hooks/useWalletMode';
import { useMobileTabletLayout } from '@/hooks/useMobileTabletLayout';

/**
 * Layout breakpoints inside the iOS/Android app (`mobileOnly`).
 * - Phone: bottom tab bar + mobile pages
 * - Tablet: same shell and pages as desktop light wallet (Sidebar + TopBar + Cards)
 */
export function useResponsiveLayout() {
  const walletMode = useWalletMode();
  const isTabletLayout = useMobileTabletLayout();
  const isPhoneLayout = walletMode.mobileOnly && !isTabletLayout;
  const useDesktopShell = !walletMode.mobileOnly || isTabletLayout;
  const useDesktopPages = useDesktopShell;

  return {
    ...walletMode,
    isTabletLayout,
    isPhoneLayout,
    useDesktopShell,
    useDesktopPages,
  };
}

/** iPhone layout only (mobile shell with tab bar). */
export function useMobilePhoneLayout(): boolean {
  return useResponsiveLayout().isPhoneLayout;
}

/** Desktop-style pages on macOS/Windows or iPad. */
export function useDesktopPageLayout(): boolean {
  return useResponsiveLayout().useDesktopPages;
}
