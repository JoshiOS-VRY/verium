import { useState, type ReactNode } from 'react';
import { ExternalLink } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { useMobilePhoneLayout } from '@/hooks/useResponsiveLayout';
import { openExternal } from '@/lib/open-external';
import { cn } from '@/lib/utils';

export function ExplorerDetailShell({
  title,
  subtitle,
  externalUrl,
  children,
}: {
  title: string;
  subtitle?: string;
  externalUrl?: string;
  children: ReactNode;
}) {
  const [openError, setOpenError] = useState<string | null>(null);
  const isPhoneLayout = useMobilePhoneLayout();

  if (isPhoneLayout) {
    return (
      <div className="mobile-page mobile-page--explorer">
        <div className="mobile-explorer-span-full mobile-panel rounded-2xl border border-border bg-bg-panel/80 p-4">
          <h2 className="text-base font-semibold text-fg">{title}</h2>
          {subtitle && (
            <p className="mt-1 text-xs leading-relaxed text-fg-muted">{subtitle}</p>
          )}
          {externalUrl && (
            <Button
              type="button"
              variant="secondary"
              className="mt-3 h-9 w-full rounded-xl text-xs"
              onClick={() => {
                setOpenError(null);
                void openExternal(externalUrl).catch((err) =>
                  setOpenError(err instanceof Error ? err.message : String(err))
                );
              }}
            >
              <ExternalLink className="h-3.5 w-3.5" />
              Open in browser
            </Button>
          )}
          {openError && <p className="mt-2 text-xs text-danger">{openError}</p>}
        </div>
        <div className="mobile-explorer-content">{children}</div>
      </div>
    );
  }

  return (
    <div className="flex min-w-0 flex-col gap-6">
      <div className="rounded-lg border border-border bg-bg-panel/80 p-5">
        <h1 className="text-2xl font-semibold tracking-tight text-fg">{title}</h1>
        {subtitle && <p className="mt-1 text-sm text-fg-muted">{subtitle}</p>}
        {externalUrl && (
          <Button
            type="button"
            variant="secondary"
            size="sm"
            className="mt-4"
            onClick={() => {
              setOpenError(null);
              void openExternal(externalUrl).catch((err) =>
                setOpenError(err instanceof Error ? err.message : String(err))
              );
            }}
          >
            <ExternalLink className="h-3.5 w-3.5" />
            Open in browser
          </Button>
        )}
        {openError && <p className="mt-2 text-xs text-danger">{openError}</p>}
      </div>
      <div className={cn('grid min-w-0 gap-6', 'lg:grid-cols-2')}>{children}</div>
    </div>
  );
}
