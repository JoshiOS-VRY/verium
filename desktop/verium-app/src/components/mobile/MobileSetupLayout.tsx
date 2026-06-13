import type { ReactNode } from 'react';
import { cn } from '@/lib/utils';

/** Full-viewport mobile shell for /setup — scroll lives here (outside AppShell). */
export function MobileSetupLayout({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <div className="mobile-setup-shell flex h-dvh max-h-dvh w-full max-w-full flex-col overflow-hidden bg-bg text-fg">
      <main
        className={cn(
          'mobile-setup-scroll min-h-0 min-w-0 flex-1 overflow-y-auto overflow-x-hidden',
          className
        )}
      >
        <div className="mobile-setup-content mx-auto w-full max-w-lg">{children}</div>
      </main>
    </div>
  );
}
