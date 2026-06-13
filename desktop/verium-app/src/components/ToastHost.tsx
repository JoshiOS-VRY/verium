import { useEffect } from 'react';
import { useLocation } from 'react-router-dom';
import { X } from 'lucide-react';
import { useWalletMode } from '@/hooks/useWalletMode';
import { isExplorerDetailPath } from '@/lib/explorer-nav';
import { useToastStore, type ToastItem } from '@/lib/toast-store';
import { cn } from '@/lib/utils';

function ToastCard({
  toast,
  onDismiss,
  mobileBottom,
}: {
  toast: ToastItem;
  onDismiss: () => void;
  mobileBottom: boolean;
}) {
  useEffect(() => {
    const ms = toast.durationMs ?? 6_000;
    const timer = window.setTimeout(onDismiss, ms);
    return () => window.clearTimeout(timer);
  }, [toast.durationMs, onDismiss]);

  return (
    <div
      role="status"
      aria-live="polite"
      className={cn(
        'pointer-events-auto flex w-full max-w-sm items-start gap-3 rounded-lg border px-4 py-3 shadow-lg backdrop-blur-sm',
        mobileBottom ? 'toast-enter-mobile' : 'toast-enter',
        toast.tone === 'success'
          ? 'border-success/40 bg-bg-panel/95'
          : 'border-border bg-bg-panel/95'
      )}
    >
      <div
        className={cn(
          'mt-0.5 h-2 w-2 shrink-0 rounded-full',
          toast.tone === 'success' ? 'bg-success' : 'bg-accent'
        )}
        aria-hidden
      />
      <div className="min-w-0 flex-1">
        <div className="text-sm font-medium text-fg">{toast.title}</div>
        {toast.description && (
          <div className="mt-0.5 whitespace-pre-line text-xs text-fg-muted">{toast.description}</div>
        )}
      </div>
      <button
        type="button"
        onClick={onDismiss}
        className="shrink-0 rounded p-0.5 text-fg-subtle transition-colors hover:bg-bg-subtle hover:text-fg"
        aria-label="Dismiss notification"
      >
        <X className="h-4 w-4" />
      </button>
    </div>
  );
}

export function ToastHost() {
  const toasts = useToastStore((s) => s.toasts);
  const dismiss = useToastStore((s) => s.dismiss);
  const { mobileOnly } = useWalletMode();
  const { pathname } = useLocation();
  const hideTabBar = mobileOnly && isExplorerDetailPath(pathname);

  if (toasts.length === 0) return null;

  return (
    <div
      className={cn(
        'pointer-events-none fixed left-4 right-4 z-[100] mx-auto flex w-full max-w-sm flex-col gap-2',
        mobileOnly ? (hideTabBar ? 'toast-host-mobile-no-tab' : 'toast-host-mobile') : 'top-4'
      )}
      aria-label="Notifications"
    >
      {toasts.map((toast) => (
        <ToastCard
          key={toast.id}
          toast={toast}
          mobileBottom={mobileOnly}
          onDismiss={() => dismiss(toast.id)}
        />
      ))}
    </div>
  );
}
