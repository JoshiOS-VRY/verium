import { useEffect, useMemo, useRef, useState } from 'react';
import { ChevronDown } from 'lucide-react';
import { cn } from '@/lib/utils';

export function SearchableAddressSelect({
  value,
  addresses,
  disabled,
  placeholder = 'Filter addresses…',
  emptyLabel = 'Select a receive address…',
  onChange,
  'aria-label': ariaLabel = 'Wallet address',
}: {
  value: string;
  addresses: string[];
  disabled?: boolean;
  placeholder?: string;
  emptyLabel?: string;
  onChange: (address: string) => void;
  'aria-label'?: string;
}) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState('');
  const rootRef = useRef<HTMLDivElement>(null);

  const options = useMemo(() => {
    const seen = new Set<string>();
    const list: string[] = [];
    const add = (addr: string) => {
      const trimmed = addr.trim();
      if (!trimmed || seen.has(trimmed)) return;
      seen.add(trimmed);
      list.push(trimmed);
    };
    if (value) add(value);
    for (const addr of addresses) add(addr);

    const q = query.trim().toLowerCase();
    const filtered = q ? list.filter((addr) => addr.toLowerCase().includes(q)) : list;
    if (value && !filtered.includes(value)) {
      return [value, ...filtered];
    }
    return filtered;
  }, [addresses, query, value]);

  useEffect(() => {
    if (!open) return;

    const onPointerDown = (event: MouseEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    };

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') setOpen(false);
    };

    document.addEventListener('mousedown', onPointerDown);
    document.addEventListener('keydown', onKeyDown);
    return () => {
      document.removeEventListener('mousedown', onPointerDown);
      document.removeEventListener('keydown', onKeyDown);
    };
  }, [open]);

  const selectAddress = (addr: string) => {
    onChange(addr);
    setQuery('');
    setOpen(false);
  };

  const triggerLabel = value.trim() || emptyLabel;

  return (
    <div ref={rootRef} className="relative min-w-0 flex-1">
      <button
        type="button"
        disabled={disabled}
        aria-expanded={open}
        aria-haspopup="listbox"
        aria-label={ariaLabel}
        onClick={() => {
          if (disabled) return;
          setOpen((v) => !v);
        }}
        className={cn(
          'flex h-9 w-full items-center gap-2 rounded-md border border-border bg-bg-panel px-3 text-left transition-colors',
          'hover:border-accent/60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent',
          disabled && 'cursor-not-allowed opacity-60'
        )}
      >
        <span
          className={cn(
            'min-w-0 flex-1 truncate font-mono text-xs',
            value ? 'text-fg' : 'text-fg-subtle'
          )}
        >
          {triggerLabel}
        </span>
        <ChevronDown
          className={cn(
            'h-4 w-4 shrink-0 text-fg-subtle transition-transform',
            open && 'rotate-180'
          )}
          aria-hidden
        />
      </button>

      {open && !disabled ? (
        <div className="absolute left-0 right-0 top-[calc(100%+0.35rem)] z-50 overflow-hidden rounded-md border border-border bg-bg-panel shadow-lg">
          <input
            type="search"
            value={query}
            autoFocus
            onChange={(e) => setQuery(e.target.value)}
            placeholder={placeholder}
            className="h-9 w-full border-b border-border bg-bg-subtle px-3 font-mono text-xs outline-none focus:border-accent"
            aria-label={`${ariaLabel} search`}
          />
          <div role="listbox" aria-label={ariaLabel} className="max-h-48 overflow-y-auto">
            {options.length === 0 ? (
              <p className="px-3 py-2 text-xs text-fg-muted">No matching addresses.</p>
            ) : (
              options.map((addr) => {
                const selected = addr === value;
                return (
                  <button
                    key={addr}
                    type="button"
                    role="option"
                    aria-selected={selected}
                    onClick={() => selectAddress(addr)}
                    className={cn(
                      'block w-full truncate px-3 py-2 text-left font-mono text-xs transition-colors',
                      'hover:bg-bg-subtle focus-visible:bg-bg-subtle focus-visible:outline-none',
                      selected && 'bg-accent/15 font-medium text-fg',
                      !selected && 'text-fg-muted'
                    )}
                  >
                    {addr}
                  </button>
                );
              })
            )}
          </div>
        </div>
      ) : null}
    </div>
  );
}
