import { Monitor, Moon, Sun } from "lucide-react";
import type { ThemeMode } from "@/lib/theme";
import { cn } from "@/lib/utils";

const THEME_OPTIONS: {
  value: ThemeMode;
  label: string;
  Icon: typeof Monitor;
}[] = [
  { value: "system", label: "Auto", Icon: Monitor },
  { value: "light", label: "Light", Icon: Sun },
  { value: "dark", label: "Dark", Icon: Moon },
];

export function ThemeSegmented({
  value,
  onChange,
}: {
  value: ThemeMode;
  onChange: (mode: ThemeMode) => Promise<void> | void;
}) {
  return (
    <div
      role="radiogroup"
      aria-label="Theme"
      className="inline-flex text-center gap-4 p-2 rounded-md border border-border bg-bg-subtle  w-full"
    >
      {THEME_OPTIONS.map(({ value: optionValue, label, Icon }) => {
        const active = value === optionValue;
        return (
          <button
            key={optionValue}
            type="button"
            role="radio"
            aria-checked={active}
            onClick={() => void onChange(optionValue)}
            className={cn(
              "w-full text-center inline-flex h-8 items-center gap-1.5 rounded px-3 text-xs font-medium transition-colors",
              active
                ? "bg-accent text-accent-fg"
                : "text-fg-muted hover:bg-bg-panel hover:text-fg",
            )}
          >
            <Icon className="h-3.5 w-3.5" />
            {label}
          </button>
        );
      })}
    </div>
  );
}
