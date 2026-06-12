import { useLocation, useNavigate } from 'react-router-dom';
import { ArrowLeft } from 'lucide-react';
import { CoinSwitcher } from '@/components/CoinSwitcher';
import { DaemonStatusBadge } from '@/components/DaemonStatusBadge';
import { LightServerBadge } from '@/components/LightServerBadge';
import { APP_ROUTE_TITLES } from '@/lib/app-nav';
import { isExplorerDetailPath } from '@/lib/explorer-nav';
import { mobileBackAriaLabel, performMobileBack } from '@/lib/mobile-back-nav';
import { useWalletMode } from '@/hooks/useWalletMode';
import { cn } from '@/lib/utils';

function mobileTitleForPath(pathname: string): string {
  if (APP_ROUTE_TITLES[pathname]) return APP_ROUTE_TITLES[pathname];
  if (pathname.startsWith('/explorer/tx/')) return 'Transaction';
  if (pathname.startsWith('/explorer/block/')) return 'Block';
  if (pathname.startsWith('/explorer/address/')) return 'Address';
  return 'Vericonomy Wallet';
}

export function MobileHeader() {
  const { pathname } = useLocation();
  const navigate = useNavigate();
  const { isLight } = useWalletMode();
  const explorerDetail = isExplorerDetailPath(pathname);
  const title = mobileTitleForPath(pathname);

  return (
    <header className="mobile-header z-30 shrink-0 border-b border-border bg-bg-subtle/95 backdrop-blur-md">
      <div className="flex min-w-0 items-center gap-2 px-3 py-1.5">
        <button
          type="button"
          onClick={() => performMobileBack(navigate, pathname)}
          className={cn(
            'inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-lg text-fg-muted',
            'transition-colors hover:bg-bg-panel hover:text-fg'
          )}
          aria-label={mobileBackAriaLabel(pathname)}
        >
          <ArrowLeft className="h-4 w-4" />
        </button>
        <div className="min-w-0 flex-1">
          <h1 className="truncate text-base font-semibold leading-tight">{title}</h1>
        </div>
        <div className="min-w-0 max-w-[42%]">
          {isLight ? <LightServerBadge /> : <DaemonStatusBadge />}
        </div>
      </div>
      {!explorerDetail && (
        <div className="min-w-0 border-t border-border/60 px-3 py-1.5">
          <CoinSwitcher />
        </div>
      )}
    </header>
  );
}
