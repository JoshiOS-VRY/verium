import { sendNativeNotification, ensureNotificationPermission } from '@/lib/notifications/client';
import { pushToast } from '@/lib/toast-store';

export interface ReceiveNotificationPayload {
  title: string;
  description?: string;
  /** When true, attempt native notification on mobile before in-app toast. */
  native?: boolean;
}

/**
 * Show a receive notification: native banner/lock screen on mobile when permitted,
 * plus in-app toast for immediate feedback while the app is open.
 */
export async function showReceiveNotification(payload: ReceiveNotificationPayload): Promise<void> {
  const { title, description, native = false } = payload;

  if (native) {
    const granted = await ensureNotificationPermission();
    if (granted) {
      await sendNativeNotification(title, description);
    }
  }

  pushToast({
    title,
    description,
    tone: 'success',
    durationMs: description ? 8_000 : 6_000,
  });
}
