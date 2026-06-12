import { APP_VERSION } from '@/lib/features';

const buildId = import.meta.env.VITE_BUILD_ID ?? 'dev';

/** Footer stamp so device installs can be verified against the latest build. */
export function MobileBuildStamp() {
  return (
    <p className="py-2 text-center text-[10px] leading-relaxed text-fg-subtle">
      v{APP_VERSION} · build {buildId}
    </p>
  );
}
