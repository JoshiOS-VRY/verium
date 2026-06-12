import { useState, type ReactNode } from 'react';
import { ExternalLink } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { openExternal } from '@/lib/open-external';

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

  return (
    <div className="flex flex-col gap-3 px-3 py-3">
      <div className="mobile-panel rounded-2xl border border-border bg-bg-panel/80 p-4">
        <h2 className="text-base font-semibold text-fg">{title}</h2>
        {subtitle && <p className="mt-1 text-xs leading-relaxed text-fg-muted">{subtitle}</p>}
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
      {children}
    </div>
  );
}
