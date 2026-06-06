import { cn } from "@/lib/utils";

type RejectTone = "good" | "warn" | "bad" | "muted";

const toneClass: Record<RejectTone, string> = {
  good: "border-success/25 bg-success/10 text-success",
  warn: "border-warning/30 bg-warning/10 text-warning",
  bad: "border-danger/30 bg-danger/10 text-danger",
  muted: "border-border bg-bg-subtle text-fg-subtle",
};

export function RejectPctBadge({
  label,
  tone,
}: {
  label: string;
  tone: RejectTone;
}) {
  return (
    <span
      className={cn(
        "inline-flex min-w-[3.25rem] items-center justify-center rounded-md border px-2 py-0.5 text-xs font-semibold tabular-nums",
        toneClass[tone],
      )}
    >
      {label}
    </span>
  );
}
