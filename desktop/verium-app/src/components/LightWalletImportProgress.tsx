import { useEffect, useState } from "react";
import { Check, Loader2 } from "lucide-react";
import { lightWalletCopy } from "@/lib/light-wallet/copy";
import { cn } from "@/lib/utils";

const STEP_ADVANCE_MS = [0, 500, 1400] as const;

type ImportPhase = "importing" | "syncing" | "done";

interface LightWalletImportProgressProps {
  phase: ImportPhase;
}

export function LightWalletImportProgress({ phase }: LightWalletImportProgressProps) {
  const [activeStep, setActiveStep] = useState(0);
  const steps = lightWalletCopy.importLightSteps;
  const isImporting = phase === "importing";
  const isSyncing = phase === "syncing";

  useEffect(() => {
    if (!isImporting) {
      setActiveStep(0);
      return;
    }
    setActiveStep(0);
    const timers = STEP_ADVANCE_MS.slice(1).map((delay, index) =>
      window.setTimeout(() => setActiveStep(index + 1), delay),
    );
    return () => timers.forEach((id) => window.clearTimeout(id));
  }, [isImporting]);

  if (phase === "done") return null;

  const visibleSteps = isSyncing
    ? [
        ...steps.map((label) => ({ label, done: true })),
        { label: lightWalletCopy.importLightSyncStep, done: false },
      ]
    : steps.map((label, index) => ({
        label,
        done: index < activeStep,
        active: index === activeStep,
      }));

  const progressPct = isSyncing
    ? 88
    : Math.min(82, 18 + activeStep * 28);

  return (
    <div
      className="flex flex-col gap-4 rounded-lg border border-accent/25 bg-accent/5 px-4 py-4"
      role="status"
      aria-live="polite"
    >
      <div className="flex items-center gap-3">
        <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-accent/15">
          <Loader2 className="h-4 w-4 animate-spin text-accent" aria-hidden />
        </div>
        <div>
          <p className="text-sm font-medium text-fg">
            {isSyncing ? "Finishing import" : "Importing light wallet"}
          </p>
          <p className="text-xs text-fg-muted">
            {isSyncing
              ? lightWalletCopy.importLightScanNote
              : "Saving locally is quick; server balance scan continues afterward."}
          </p>
        </div>
      </div>

      <div className="h-1.5 overflow-hidden rounded-full bg-border">
        <div
          className={cn(
            "h-full rounded-full bg-accent transition-all duration-700 ease-out",
            isSyncing && "animate-pulse",
          )}
          style={{ width: `${progressPct}%` }}
        />
      </div>

      <ol className="flex flex-col gap-2">
        {visibleSteps.map((step, index) => {
          const done = "done" in step && step.done;
          const active = "active" in step && step.active;
          return (
            <li
              key={`${step.label}-${index}`}
              className={cn(
                "flex items-center gap-2 text-xs transition-colors",
                done ? "text-success" : active ? "text-fg" : "text-fg-subtle",
              )}
            >
              {done ? (
                <Check className="h-3.5 w-3.5 shrink-0" aria-hidden />
              ) : active || (isSyncing && index === visibleSteps.length - 1) ? (
                <Loader2
                  className="h-3.5 w-3.5 shrink-0 animate-spin text-accent"
                  aria-hidden
                />
              ) : (
                <span className="h-3.5 w-3.5 shrink-0 rounded-full border border-border" />
              )}
              <span>{step.label}</span>
            </li>
          );
        })}
      </ol>
    </div>
  );
}
