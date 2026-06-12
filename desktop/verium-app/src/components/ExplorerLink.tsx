import { useCallback, type ReactNode } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { ExternalLink } from 'lucide-react';
import { useWalletMode } from '@/hooks/useWalletMode';
import { useActiveCoin } from '@/lib/coin/context';
import type { CoinId } from '@/lib/coin/profile';
import {
  buildAddressExplorerUrl,
  buildBlockExplorerUrl,
  buildTxExplorerUrl,
  effectiveAddressExplorerTemplate,
  effectiveBlockExplorerTemplate,
  effectiveTxExplorerTemplate,
  explorerHome,
} from '@/lib/explorer-links';
import { explorerAddressPath, explorerBlockPath, explorerTxPath } from '@/lib/explorer-nav';
import { useUserPreferences, type UserPreferences } from '@/lib/user-preferences';
import { openExternal } from '@/lib/open-external';
import { cn } from '@/lib/utils';

export type ExplorerTarget =
  | { kind: 'home' }
  | { kind: 'tx'; txid: string }
  | { kind: 'block'; hashOrHeight: string | number }
  | { kind: 'address'; address: string }
  | { kind: 'raw'; url: string };

/** Press feedback for explorer list cards (mobile-friendly). */
export const explorerCardTapClass = cn(
  'block w-full min-w-0 max-w-full text-left',
  'cursor-pointer transition-[transform,background-color,opacity] duration-150 ease-out',
  'active:scale-[0.98] active:opacity-95',
  'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/45 focus-visible:ring-offset-2 focus-visible:ring-offset-bg'
);

function resolveExplorerExternalUrl(
  target: ExplorerTarget,
  coin: CoinId,
  prefs: UserPreferences
): string {
  const txTemplate = effectiveTxExplorerTemplate(coin, prefs.explorer_tx_url_template);
  const blockTemplate = effectiveBlockExplorerTemplate(coin, prefs.explorer_block_url_template);
  const addressTemplate = effectiveAddressExplorerTemplate(
    coin,
    prefs.explorer_address_url_template
  );
  switch (target.kind) {
    case 'home':
      return explorerHome(coin);
    case 'tx':
      return buildTxExplorerUrl(coin, txTemplate, target.txid);
    case 'block':
      return buildBlockExplorerUrl(coin, blockTemplate, target.hashOrHeight);
    case 'address':
      return buildAddressExplorerUrl(coin, addressTemplate, target.address);
    case 'raw':
      return target.url;
  }
}

export function useExplorerNavigate(coinProp?: CoinId) {
  const navigate = useNavigate();
  const { mobileOnly } = useWalletMode();
  const activeCoin = useActiveCoin();
  const coin = coinProp ?? activeCoin;
  const { prefs } = useUserPreferences();

  return useCallback(
    (target: ExplorerTarget) => {
      const inAppPath = mobileOnly ? inAppExplorerPath(target) : null;
      if (inAppPath) {
        navigate(inAppPath);
        return;
      }
      void openExternal(resolveExplorerExternalUrl(target, coin, prefs));
    },
    [coin, mobileOnly, navigate, prefs]
  );
}

interface ExplorerLinkProps {
  target: ExplorerTarget;
  coin?: CoinId;
  label?: ReactNode;
  className?: string;
  showIcon?: boolean;
  title?: string;
}

function inAppExplorerPath(target: ExplorerTarget): string | null {
  switch (target.kind) {
    case 'tx':
      return explorerTxPath(target.txid);
    case 'block':
      return explorerBlockPath(target.hashOrHeight);
    case 'address':
      return explorerAddressPath(target.address);
    case 'home':
    case 'raw':
      return null;
  }
}

export function ExplorerLink({
  target,
  coin: coinProp,
  label = 'View on explorer',
  className,
  showIcon = true,
  title,
}: ExplorerLinkProps) {
  const { mobileOnly } = useWalletMode();
  const activeCoin = useActiveCoin();
  const coin = coinProp ?? activeCoin;
  const navigateExplorer = useExplorerNavigate(coin);
  const inAppPath = mobileOnly ? inAppExplorerPath(target) : null;
  const useInApp = Boolean(inAppPath);

  return (
    <button
      type="button"
      title={title ?? (typeof label === 'string' ? label : 'View on explorer')}
      onClick={() => navigateExplorer(target)}
      className={cn(
        'inline-flex items-center gap-1 text-xs text-accent underline-offset-2 hover:underline',
        className
      )}
    >
      {label}
      {showIcon && !useInApp && <ExternalLink className="h-3 w-3 shrink-0 opacity-70 truncate" />}
    </button>
  );
}

export function ExplorerCardLink({
  target,
  coin: coinProp,
  className,
  children,
  ariaLabel,
}: {
  target: ExplorerTarget;
  coin?: CoinId;
  className?: string;
  children: ReactNode;
  ariaLabel: string;
}) {
  const { mobileOnly } = useWalletMode();
  const activeCoin = useActiveCoin();
  const coin = coinProp ?? activeCoin;
  const navigateExplorer = useExplorerNavigate(coin);
  const inAppPath = mobileOnly ? inAppExplorerPath(target) : null;

  if (inAppPath) {
    return (
      <Link to={inAppPath} aria-label={ariaLabel} className={cn(explorerCardTapClass, className)}>
        {children}
      </Link>
    );
  }

  return (
    <button
      type="button"
      aria-label={ariaLabel}
      onClick={() => navigateExplorer(target)}
      className={cn(explorerCardTapClass, className)}
    >
      {children}
    </button>
  );
}

/** Tappable card when a nested link must remain interactive (e.g. block # on a tx row). */
export function TappableExplorerSurface({
  onActivate,
  className,
  children,
  ariaLabel,
}: {
  onActivate: () => void;
  className?: string;
  children: ReactNode;
  ariaLabel: string;
}) {
  return (
    <article
      role="button"
      tabIndex={0}
      aria-label={ariaLabel}
      onClick={onActivate}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          onActivate();
        }
      }}
      className={cn(explorerCardTapClass, className)}
    >
      {children}
    </article>
  );
}
