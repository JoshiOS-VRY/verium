import { Loader2, Radio } from 'lucide-react';
import type { WalletInfo } from '@/lib/rpc/client';
import { lightWalletCopy } from '@/lib/light-wallet/copy';
import { cn } from '@/lib/utils';

function scanPhaseLabel(phase: WalletInfo['light_scan_phase']): string {
  switch (phase) {
    case 'precache':
      return lightWalletCopy.scanPhasePrecache;
    case 'external':
      return lightWalletCopy.scanPhaseExternal;
    case 'internal':
      return lightWalletCopy.scanPhaseInternal;
    default:
      return lightWalletCopy.balanceCheckingAddresses;
  }
}

function formatScanPercent(progress: number): string {
  const pct = Math.round(Math.max(0, Math.min(1, progress)) * 100);
  return lightWalletCopy.scanProgress.replace('{percent}', String(pct));
}

export function LightWalletSyncStatus({
  wallet,
  unlocked,
  isLight,
  refreshing = false,
  className,
  compact = false,
}: {
  wallet: WalletInfo;
  unlocked: boolean;
  isLight: boolean;
  /** Pull-to-refresh or explicit user refresh in flight. */
  refreshing?: boolean;
  className?: string;
  compact?: boolean;
}) {
  if (!isLight || !unlocked) return null;

  const lightSyncing = wallet.light_syncing === true;
  const balanceSyncing = wallet.light_balance_syncing === true;
  const balanceReady = wallet.light_balance_ready === true;
  const setupSyncing = wallet.light_setup_syncing === true || (!balanceReady && unlocked);
  const scanProgress = wallet.light_scan_progress ?? 0;
  const showDeterminateProgress = lightSyncing && scanProgress < 1;
  const showSetupProgress = setupSyncing || lightSyncing || balanceSyncing;

  if (refreshing) {
    return (
      <div
        className={cn(
          'flex items-center justify-center gap-1.5 text-[10px] font-medium uppercase tracking-wide text-amber-600 dark:text-amber-400',
          className
        )}
      >
        <Loader2 className="h-3 w-3 animate-spin" aria-hidden />
        {lightWalletCopy.balanceRefreshing}
      </div>
    );
  }

  if (showSetupProgress) {
    const phaseLabel = lightSyncing
      ? scanPhaseLabel(wallet.light_scan_phase)
      : lightWalletCopy.balanceCheckingAddresses;
    const indeterminate = !showDeterminateProgress;

    return (
      <div className={cn('space-y-1.5', className)}>
        <p className="flex items-center justify-center gap-1.5 text-[10px] font-medium uppercase tracking-wide text-amber-600 dark:text-amber-400">
          <Loader2 className="h-3 w-3 animate-spin" aria-hidden />
          {phaseLabel}
          {!compact && showDeterminateProgress && (
            <span className="normal-case tracking-normal text-fg-muted">
              {formatScanPercent(scanProgress)}
            </span>
          )}
        </p>
        <div
          className="mx-auto h-1 max-w-[12rem] overflow-hidden rounded-full bg-amber-500/15"
          role="progressbar"
          aria-valuenow={showDeterminateProgress ? Math.round(scanProgress * 100) : undefined}
          aria-valuemin={0}
          aria-valuemax={100}
          aria-label={phaseLabel}
          aria-busy={indeterminate}
        >
          <div
            className={cn(
              'h-full rounded-full bg-amber-500',
              indeterminate
                ? 'light-sync-progress-indeterminate w-2/5'
                : 'transition-[width] duration-500 ease-out'
            )}
            style={
              indeterminate
                ? undefined
                : { width: `${Math.max(8, scanProgress * 100)}%` }
            }
          />
        </div>
      </div>
    );
  }

  if (balanceReady) {
    return (
      <p
        className={cn(
          'flex items-center justify-center gap-1.5 text-[10px] font-medium uppercase tracking-wide text-emerald-600 dark:text-emerald-400',
          className
        )}
      >
        <Radio className="h-3 w-3" aria-hidden />
        {lightWalletCopy.balanceUpToDate}
      </p>
    );
  }

  return null;
}
