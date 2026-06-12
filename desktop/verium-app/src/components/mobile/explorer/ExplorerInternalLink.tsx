import type { ReactNode } from 'react';
import { Link } from 'react-router-dom';
import { cn } from '@/lib/utils';

export function ExplorerInternalLink({
  to,
  children,
  className,
  mono,
}: {
  to: string;
  children: ReactNode;
  className?: string;
  mono?: boolean;
}) {
  return (
    <Link
      to={to}
      className={cn(
        'text-accent underline-offset-2 hover:underline',
        mono && 'font-mono text-[11px] break-all',
        className
      )}
    >
      {children}
    </Link>
  );
}
