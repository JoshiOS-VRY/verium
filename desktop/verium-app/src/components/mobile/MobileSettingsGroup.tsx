import { type ReactNode, useState } from 'react';
import { ChevronDown } from 'lucide-react';
import { cn } from '@/lib/utils';

export function MobileSettingsGroup({
  title,
  description,
  defaultOpen = true,
  children,
  className,
}: {
  title: string;
  description?: string;
  defaultOpen?: boolean;
  children: ReactNode;
  className?: string;
}) {
  const [open, setOpen] = useState(defaultOpen);

  return (
    <section
      className={cn(
        'mobile-panel overflow-hidden rounded-2xl border border-border bg-bg-panel shadow-sm',
        className
      )}
    >
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        className="flex w-full items-start justify-between gap-3 px-4 py-4 text-left"
      >
        <div className="min-w-0 flex-1">
          <h2 className="text-sm font-semibold text-fg">{title}</h2>
          {description ? (
            <p className="mt-1 text-xs leading-relaxed text-fg-subtle">{description}</p>
          ) : null}
        </div>
        <ChevronDown
          className={cn(
            'mt-0.5 h-5 w-5 shrink-0 text-fg-muted transition-transform',
            open && 'rotate-180'
          )}
        />
      </button>
      {open ? <div className="border-t border-border/60 px-4 py-4">{children}</div> : null}
    </section>
  );
}
