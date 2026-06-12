import { useEffect, useState } from 'react';
import { Bell } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { useNotificationPermission } from '@/hooks/useNotificationPermission';
import { useWalletMode } from '@/hooks/useWalletMode';
import { useUserPreferences } from '@/lib/user-preferences';

interface NotificationPermissionPromptViewProps {
  busy: boolean;
  onAllow: () => void;
  onDismiss: () => void;
}

/** iOS-style sheet to request notification permission for receive alerts. */
export function NotificationPermissionPromptView({
  busy,
  onAllow,
  onDismiss,
}: NotificationPermissionPromptViewProps) {
  return (
    <div
      className="fixed inset-0 z-[65] flex items-end justify-center bg-black/45 p-4 backdrop-blur-[2px] sm:items-center"
      role="dialog"
      aria-modal="true"
      aria-labelledby="notification-permission-title"
    >
      <div className="w-full max-w-sm rounded-2xl border border-border bg-bg-panel p-5 shadow-2xl">
        <div className="flex flex-col items-center text-center">
          <div className="mb-4 flex h-16 w-16 items-center justify-center rounded-2xl bg-accent/10">
            <Bell className="h-8 w-8 text-accent" aria-hidden />
          </div>
          <h2 id="notification-permission-title" className="text-lg font-semibold text-fg">
            Get notified when you receive coins?
          </h2>
          <p className="mt-2 text-sm leading-relaxed text-fg-muted">
            Allow notifications to see incoming VRM and VRC on your lock screen and as banners when
            you&apos;re not in the app.
          </p>
        </div>
        <div className="mt-5 flex flex-col gap-2">
          <Button className="h-11 w-full rounded-xl" disabled={busy} onClick={onAllow}>
            {busy ? 'Requesting…' : 'Allow notifications'}
          </Button>
          <Button
            type="button"
            variant="ghost"
            className="h-11 w-full rounded-xl text-fg-muted"
            disabled={busy}
            onClick={onDismiss}
          >
            Not Now
          </Button>
        </div>
      </div>
    </div>
  );
}

/**
 * One-time mobile prompt for iOS notification permission (banner + lock screen).
 * Shown after wallet setup / first dashboard visit when alerts are enabled in prefs.
 */
export function NotificationPermissionOfferHost() {
  const { mobileOnly } = useWalletMode();
  const prefs = useUserPreferences((s) => s.prefs);
  const prefsLoaded = useUserPreferences((s) => s.loaded);
  const updatePrefs = useUserPreferences((s) => s.update);
  const notif = useNotificationPermission();
  const [open, setOpen] = useState(false);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (!mobileOnly || !prefsLoaded || notif.loading) return;
    if (!notif.available || notif.granted) return;
    if (prefs.notifications_prompt_dismissed) return;
    const wantsNotify =
      prefs.notify_on_vrm_received !== false || prefs.notify_on_vrc_received !== false;
    if (!wantsNotify) return;

    const timer = window.setTimeout(() => setOpen(true), 1_200);
    return () => window.clearTimeout(timer);
  }, [
    mobileOnly,
    prefsLoaded,
    notif.loading,
    notif.available,
    notif.granted,
    prefs.notifications_prompt_dismissed,
    prefs.notify_on_vrm_received,
    prefs.notify_on_vrc_received,
  ]);

  if (!open) return null;

  return (
    <NotificationPermissionPromptView
      busy={busy}
      onAllow={() => {
        setBusy(true);
        void notif.request().finally(() => {
          setBusy(false);
          setOpen(false);
          void updatePrefs({ notifications_prompt_dismissed: true });
        });
      }}
      onDismiss={() => {
        setOpen(false);
        void updatePrefs({ notifications_prompt_dismissed: true });
      }}
    />
  );
}
