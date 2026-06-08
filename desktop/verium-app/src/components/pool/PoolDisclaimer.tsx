import { POOL_DISCLAIMER } from "@/lib/verium-pool";

export function PoolDisclaimer() {
  return (
    <p
      className="rounded-lg border border-border bg-bg-subtle/50 px-4 py-3 text-xs leading-relaxed text-fg-muted"
      role="note"
    >
      {POOL_DISCLAIMER}
    </p>
  );
}
