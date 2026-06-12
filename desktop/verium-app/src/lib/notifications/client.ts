/** Native push / local notifications (iOS banner + lock screen). */

export type NotificationPermission = 'granted' | 'denied' | 'default';

export async function isNativeNotificationsAvailable(): Promise<boolean> {
  try {
    const { isPermissionGranted } = await import('@tauri-apps/plugin-notification');
    await isPermissionGranted();
    return true;
  } catch {
    return false;
  }
}

export async function getNotificationPermission(): Promise<NotificationPermission> {
  try {
    const { isPermissionGranted } = await import('@tauri-apps/plugin-notification');
    const granted = await isPermissionGranted();
    return granted ? 'granted' : 'denied';
  } catch {
    return 'denied';
  }
}

/** Request iOS/Android notification permission (banner + lock screen). */
export async function requestNotificationPermission(): Promise<NotificationPermission> {
  try {
    const { isPermissionGranted, requestPermission } = await import(
      '@tauri-apps/plugin-notification'
    );
    if (await isPermissionGranted()) {
      return 'granted';
    }
    const result = await requestPermission();
    return result === 'granted' ? 'granted' : 'denied';
  } catch {
    return 'denied';
  }
}

export async function ensureNotificationPermission(): Promise<boolean> {
  const perm = await getNotificationPermission();
  if (perm === 'granted') return true;
  const requested = await requestNotificationPermission();
  return requested === 'granted';
}

export async function sendNativeNotification(title: string, body?: string): Promise<void> {
  try {
    const { sendNotification } = await import('@tauri-apps/plugin-notification');
    await sendNotification({
      title,
      body: body ?? undefined,
      sound: 'default',
    });
  } catch {
    // Plugin unavailable (desktop webview) — caller may fall back to toast.
  }
}
