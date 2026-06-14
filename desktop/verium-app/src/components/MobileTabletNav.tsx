import { NavLink } from 'react-router-dom';
import { useMobileNavTabs } from '@/hooks/useMobileNavTabs';
import { cn } from '@/lib/utils';

export function MobileTabletNav() {
  const tabs = useMobileNavTabs();

  return (
    <nav
      className="mobile-tablet-nav shrink-0 flex-col border-r border-border bg-bg-subtle/95 backdrop-blur-md"
      aria-label="Primary"
    >
      <div className="mobile-tablet-nav-brand px-3 pb-1 pt-2">
        <p className="text-[10px] font-semibold uppercase tracking-wider text-fg-subtle">
          Vericonomy
        </p>
        <p className="text-xs font-bold text-fg">Wallet</p>
      </div>
      <div className="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto px-2 py-3">
        {tabs.map(({ to, shortLabel, label, icon: Icon }) => (
          <NavLink
            key={to}
            to={to}
            className={({ isActive }) =>
              cn(
                'mobile-tablet-nav-item flex min-w-0 flex-col items-center gap-1 rounded-xl px-2 py-2.5 text-[11px] font-medium leading-tight transition-colors',
                isActive
                  ? 'bg-accent/15 text-accent'
                  : 'text-fg-muted hover:bg-bg-panel hover:text-fg'
              )
            }
          >
            <Icon className="h-[22px] w-[22px] shrink-0" aria-hidden />
            <span className="max-w-full truncate text-center">{shortLabel ?? label}</span>
          </NavLink>
        ))}
      </div>
    </nav>
  );
}
