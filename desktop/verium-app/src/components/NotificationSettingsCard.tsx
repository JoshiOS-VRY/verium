import { Bell, BellOff } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { MobileSettingsGroup } from '@/components/mobile/MobileSettingsGroup';
import { useNotificationPermission } from '@/hooks/useNotificationPermission';
import { unlockReceivedVrmAudio, playReceivedVrmSound } from '@/lib/received-vrm-sound';
import { useUserPreferences } from '@/lib/user-preferences';

/** Mobile notification toggles + iOS permission for banner / lock screen alerts. */
export function NotificationSettingsCard({ mobileLayout = false }: { mobileLayout?: boolean }) {
  const prefs = useUserPreferences((s) => s.prefs);
  const updatePrefs = useUserPreferences((s) => s.update);
  const notif = useNotificationPermission();

  const vrmEnabled = prefs.notify_on_vrm_received !== false;
  const vrcEnabled = prefs.notify_on_vrc_received !== false;

  const permissionHint = notif.available
    ? notif.granted
      ? 'Banner and lock screen alerts are enabled.'
      : 'Allow notifications so incoming VRM and VRC appear on your lock screen and as banners.'
    : 'In-app alerts are shown while the app is open.';

  const content = (
    <div className="flex flex-col gap-3">
      <label className={mobileLayout ? 'mobile-checkbox-row' : 'flex items-center gap-2 text-sm'}>
        <input
          type="checkbox"
          checked={vrmEnabled}
          onChange={(e) => {
            const checked = e.target.checked;
            void unlockReceivedVrmAudio();
            void updatePrefs({ notify_on_vrm_received: checked });
            if (checked) void playReceivedVrmSound();
            if (checked && notif.available && !notif.granted) {
              void notif.request();
              void updatePrefs({ notifications_prompt_dismissed: true });
            }
          }}
        />
        <span>Notify when VRM is received</span>
      </label>
      <label className={mobileLayout ? 'mobile-checkbox-row' : 'flex items-center gap-2 text-sm'}>
        <input
          type="checkbox"
          checked={vrcEnabled}
          onChange={(e) => {
            const checked = e.target.checked;
            void updatePrefs({ notify_on_vrc_received: checked });
            if (checked && notif.available && !notif.granted) {
              void notif.request();
              void updatePrefs({ notifications_prompt_dismissed: true });
            }
          }}
        />
        <span>Notify when VRC is received</span>
      </label>

      {notif.available && (
        <div className="rounded-xl border border-border bg-bg-subtle/40 p-3">
          <div className="flex items-start gap-2 text-xs text-fg-muted">
            {notif.granted ? (
              <Bell className="mt-0.5 h-3.5 w-3.5 shrink-0 text-success" />
            ) : (
              <BellOff className="mt-0.5 h-3.5 w-3.5 shrink-0 text-fg-subtle" />
            )}
            <p className="leading-relaxed">{permissionHint}</p>
          </div>
          {!notif.granted && (
            <Button
              type="button"
              variant="secondary"
              className="mt-3 h-10 w-full rounded-xl"
              disabled={notif.loading}
              onClick={() => void notif.request()}
            >
              Allow lock screen & banner alerts
            </Button>
          )}
        </div>
      )}
    </div>
  );

  if (mobileLayout) {
    return (
      <MobileSettingsGroup
        title="Notifications"
        description="Incoming VRM and VRC alerts while the app runs in the background."
        defaultOpen={!notif.granted}
      >
        {content}
      </MobileSettingsGroup>
    );
  }

  return content;
}
