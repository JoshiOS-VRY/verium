import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

export interface MobileSegmentOption<T extends string> {
  value: T;
  label: string;
  icon?: ReactNode;
}

export function MobileSegmented<T extends string>({
  value,
  options,
  onChange,
  ariaLabel,
  className,
}: {
  value: T;
  options: MobileSegmentOption<T>[];
  onChange: (value: T) => void;
  ariaLabel: string;
  className?: string;
}) {
  return (
    <div
      role="radiogroup"
      aria-label={ariaLabel}
      className={cn(
        "flex w-full min-w-0 rounded-xl border border-border bg-bg-subtle p-1",
        className,
      )}
    >
      {options.map((opt) => (
        <button
          key={opt.value}
          type="button"
          role="radio"
          aria-checked={value === opt.value}
          onClick={() => onChange(opt.value)}
          className={cn(
            "flex min-w-0 flex-1 items-center justify-center gap-1.5 rounded-lg px-2 py-2.5 text-xs font-semibold transition-colors",
            value === opt.value
              ? "bg-accent text-accent-fg shadow-sm"
              : "text-fg-muted active:bg-bg-panel",
          )}
        >
          {opt.icon}
          <span className="truncate">{opt.label}</span>
        </button>
      ))}
    </div>
  );
}
