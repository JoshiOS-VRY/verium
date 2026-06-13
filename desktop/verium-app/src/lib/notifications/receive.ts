import { pushToast } from '@/lib/toast-store';

export interface ReceiveNotificationPayload {
  title: string;
  description?: string;
}

/**
 * Foreground receive alert: in-app toast only.
 * Background / lock screen uses remote APNs (see useRemotePushRegistration).
 */
export async function showReceiveNotification(payload: ReceiveNotificationPayload): Promise<void> {
  const { title, description } = payload;

  pushToast({
    title,
    description,
    tone: 'success',
    durationMs: description ? 8_000 : 6_000,
  });
}
