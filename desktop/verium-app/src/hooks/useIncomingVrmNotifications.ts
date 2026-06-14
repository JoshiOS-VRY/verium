import { useEffect } from 'react';
import { subscribeIncomingVrm } from '@/hooks/useIncomingVrmWatcher';
import { playReceivedVrmSound } from '@/lib/received-vrm-sound';
import { showReceiveNotification } from '@/lib/notifications/receive';
import { formatReceiveBatchMessage } from '@/lib/notifications/receive-batch-message';
import { useUserPreferences } from '@/lib/user-preferences';

/** Shows toast + plays chime when incoming VRM is detected (if enabled). */
export function useIncomingVrmNotifications(): void {
  const enabled = useUserPreferences((s) => s.prefs.notify_on_vrm_received !== false);

  useEffect(() => {
    if (!enabled) return;

    return subscribeIncomingVrm((batch) => {
      const { title, description } = formatReceiveBatchMessage(
        batch.events,
        batch.totalAmount,
        'VRM'
      );
      void showReceiveNotification({
        title,
        description: description || undefined,
      });
      void playReceivedVrmSound();
    });
  }, [enabled]);
}
