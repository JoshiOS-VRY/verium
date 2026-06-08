import type { ReactNode } from "react";

/** Small labelled tile used across onboarding step intros. */
export function FeatureTile({
  icon,
  title,
  body,
}: {
  icon: ReactNode;
  title: string;
  body: string;
}) {
  return (
    <div className="flex flex-col gap-1 rounded-md border border-border bg-bg-subtle p-3 text-xs">
      <div className="flex items-center gap-1.5 text-fg">
        {icon}
        <span className="font-medium">{title}</span>
      </div>
      <p className="text-fg-muted">{body}</p>
    </div>
  );
}
