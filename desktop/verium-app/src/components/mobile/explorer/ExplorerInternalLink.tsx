import type { ReactNode } from 'react';
import { Link } from 'react-router-dom';
import { cn } from '@/lib/utils';

export function ExplorerInternalLink({
  to,
  children,
  className,
  mono,
  onClick,
}: {
  to: string;
  children: ReactNode;
  className?: string;
  mono?: boolean;
  onClick?: (event: React.MouseEvent<HTMLAnchorElement>) => void;
}) {
  return (
    <Link
      to={to}
      onClick={onClick}
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
