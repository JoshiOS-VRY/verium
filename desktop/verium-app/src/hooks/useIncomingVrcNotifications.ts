import { useEffect } from 'react';
import { subscribeIncomingVrc } from '@/hooks/useIncomingVrcWatcher';
import { playReceivedVrmSound } from '@/lib/received-vrm-sound';
import { showReceiveNotification } from '@/lib/notifications/receive';
import { formatReceiveBatchMessage } from '@/lib/notifications/receive-batch-message';
import { useUserPreferences } from '@/lib/user-preferences';

/** Shows toast + plays chime when incoming VRC is detected (if enabled). */
export function useIncomingVrcNotifications(): void {
  const enabled = useUserPreferences((s) => s.prefs.notify_on_vrc_received !== false);

  useEffect(() => {
    if (!enabled) return;

    return subscribeIncomingVrc((batch) => {
      const { title, description } = formatReceiveBatchMessage(batch.events, batch.totalAmount, 'VRC');
      void showReceiveNotification({
        title,
        description: description || undefined,
      });
      void playReceivedVrmSound();
    });
  }, [enabled]);
}
