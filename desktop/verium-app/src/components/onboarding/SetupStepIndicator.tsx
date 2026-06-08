import { CheckCircle2, Circle } from "lucide-react";

export interface SetupStep {
  id: string;
  label: string;
}

/** Horizontal "step 1 / step 2 / ..." progress indicator for the wizard. */
export function SetupStepIndicator({
  steps,
  currentId,
}: {
  steps: SetupStep[];
  currentId: string;
}) {
  const currentIdx = steps.findIndex((x) => x.id === currentId);
  return (
    <ol className="flex flex-wrap items-center gap-3 text-xs text-fg-subtle">
      {steps.map((s, idx) => {
        const reached = idx <= currentIdx;
        return (
          <li key={s.id} className="flex items-center gap-2">
            {reached ? (
              <CheckCircle2 className="h-3.5 w-3.5 text-success" />
            ) : (
              <Circle className="h-3.5 w-3.5" />
            )}
            <span className={reached ? "text-fg" : ""}>{s.label}</span>
            {idx < steps.length - 1 && (
              <span className="text-fg-subtle">/</span>
            )}
          </li>
        );
      })}
    </ol>
  );
}
