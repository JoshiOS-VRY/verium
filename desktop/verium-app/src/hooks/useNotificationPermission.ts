import { useCallback, useEffect, useState } from 'react';
import {
  getNotificationPermission,
  isNativeNotificationsAvailable,
  requestNotificationPermission,
  type NotificationPermission,
} from '@/lib/notifications/client';

export function useNotificationPermission() {
  const [available, setAvailable] = useState(false);
  const [permission, setPermission] = useState<NotificationPermission>('default');
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    setLoading(true);
    const ok = await isNativeNotificationsAvailable();
    setAvailable(ok);
    if (ok) {
      setPermission(await getNotificationPermission());
    } else {
      setPermission('denied');
    }
    setLoading(false);
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const request = useCallback(async () => {
    const result = await requestNotificationPermission();
    setPermission(result);
    return result;
  }, []);

  return {
    available,
    permission,
    loading,
    granted: permission === 'granted',
    refresh,
    request,
  };
}
