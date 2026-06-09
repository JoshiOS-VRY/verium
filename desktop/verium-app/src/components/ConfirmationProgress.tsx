import { Check } from "lucide-react";
import {
  blocksUntilSpendable,
  confirmationProgress,
  confirmationStatusLabel,
  isFullyConfirmed,
  requiredConfirmations,
} from "@/lib/confirmations";
import { cn } from "@/lib/utils";

interface ConfirmationProgressProps {
  confirmations: number;
  category: string;
  className?: string;
}

const SIZE = 32;
const STROKE = 3.5;
const RADIUS = (SIZE - STROKE) / 2 - 1;
const CENTER = SIZE / 2;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

export function ConfirmationProgress({
  confirmations,
  category,
  className,
}: ConfirmationProgressProps) {
  const required = requiredConfirmations(category);
  const progress = confirmationProgress(confirmations, required);
  const complete = isFullyConfirmed(confirmations, required);
  const remaining = blocksUntilSpendable(confirmations, category);
  const offset = CIRCUMFERENCE * (1 - progress);

  const ringColor = complete
    ? "stroke-success"
    : progress > 0
      ? category === "immature" || category === "generate"
        ? "stroke-warning"
        : "stroke-accent"
      : "stroke-fg-subtle";

  return (
    <div
      className={cn("inline-flex items-center justify-end gap-2", className)}
      title={confirmationStatusLabel(confirmations, category)}
    >
      <div
        className="relative shrink-0"
        style={{ width: SIZE, height: SIZE }}
      >
        <svg
          width={SIZE}
          height={SIZE}
          viewBox={`0 0 ${SIZE} ${SIZE}`}
          className="block"
          aria-hidden
        >
          <circle
            cx={CENTER}
            cy={CENTER}
            r={RADIUS}
            fill="none"
            className="stroke-border"
            strokeWidth={STROKE}
          />
          <circle
            cx={CENTER}
            cy={CENTER}
            r={RADIUS}
            fill="none"
            className={cn(ringColor, "transition-[stroke-dashoffset] duration-300")}
            strokeWidth={STROKE}
            strokeLinecap="round"
            strokeDasharray={CIRCUMFERENCE}
            strokeDashoffset={offset}
            transform={`rotate(-90 ${CENTER} ${CENTER})`}
          />
        </svg>
        {complete ? (
          <Check
            className="pointer-events-none absolute left-1/2 top-1/2 h-3.5 w-3.5 -translate-x-1/2 -translate-y-1/2 text-success"
            strokeWidth={3}
            aria-hidden
          />
        ) : null}
      </div>
      <span className="min-w-[4rem] text-right text-xs tabular-nums text-fg-muted">
        {confirmations}/{required}
        {remaining > 0 && (
          <span className="block text-[10px] text-fg-subtle">
            −{remaining} left
          </span>
        )}
      </span>
    </div>
  );
}
