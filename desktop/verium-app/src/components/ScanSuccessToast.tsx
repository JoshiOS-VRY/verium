import { useEffect } from 'react';
import { cn } from '@/lib/utils';

export function ScanSuccessToast({
  message,
  durationMs = 1000,
  onDone,
  className,
}: {
  message: string;
  durationMs?: number;
  onDone: () => void;
  className?: string;
}) {
  useEffect(() => {
    const timer = window.setTimeout(onDone, durationMs);
    return () => window.clearTimeout(timer);
  }, [durationMs, onDone]);

  return (
    <div
      className={cn(
        'fixed inset-0 z-[60] flex items-center justify-center bg-black/45 p-6 backdrop-blur-[2px]',
        className
      )}
      role="status"
      aria-live="polite"
      aria-atomic="true"
    >
      <div className="scan-success-toast flex flex-col items-center gap-3 rounded-2xl border border-success/35 bg-bg-panel/95 px-8 py-7 shadow-2xl backdrop-blur-md">
        <div className="scan-success-check-wrap" aria-hidden>
          <svg
            className="h-16 w-16"
            viewBox="0 0 64 64"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <circle
              className="scan-success-circle"
              cx="32"
              cy="32"
              r="28"
              stroke="var(--success)"
              strokeWidth="3"
              fill="color-mix(in srgb, var(--success) 12%, transparent)"
            />
            <path
              className="scan-success-check"
              d="M20 33.5L28.5 42L44.5 24"
              stroke="var(--success)"
              strokeWidth="3.5"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </div>
        <p className="text-center text-sm font-semibold text-fg">{message}</p>
      </div>
    </div>
  );
}
