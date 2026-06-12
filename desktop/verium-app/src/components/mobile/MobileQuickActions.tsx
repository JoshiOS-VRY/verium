import { Link } from 'react-router-dom';
import { ArrowDownLeft, ArrowUpRight } from 'lucide-react';
import { cn } from '@/lib/utils';

export function MobileQuickActions({ className }: { className?: string }) {
  return (
    <div className={cn('grid grid-cols-2 gap-3', className)}>
      <Link
        to="/transactions"
        state={{ mobileActivityView: 'send' }}
        className="mobile-action-tile flex flex-col items-center gap-2 rounded-2xl border border-border bg-bg-panel px-4 py-5 text-center shadow-sm transition-colors active:bg-bg-subtle"
      >
        <span className="flex h-11 w-11 items-center justify-center rounded-full bg-accent/15 text-accent">
          <ArrowUpRight className="h-5 w-5" />
        </span>
        <span className="text-sm font-semibold text-fg">Send</span>
      </Link>
      <Link
        to="/transactions"
        state={{ mobileActivityView: 'receive' }}
        className="mobile-action-tile flex flex-col items-center gap-2 rounded-2xl border border-border bg-bg-panel px-4 py-5 text-center shadow-sm transition-colors active:bg-bg-subtle"
      >
        <span className="flex h-11 w-11 items-center justify-center rounded-full bg-success/15 text-success">
          <ArrowDownLeft className="h-5 w-5" />
        </span>
        <span className="text-sm font-semibold text-fg">Receive</span>
      </Link>
    </div>
  );
}
