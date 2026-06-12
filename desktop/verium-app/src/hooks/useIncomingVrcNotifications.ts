import { useEffect } from 'react';
import { subscribeIncomingVrc, type IncomingVrcBatch } from '@/hooks/useIncomingVrcWatcher';
import { useWalletMode } from '@/hooks/useWalletMode';
import { playReceivedVrmSound } from '@/lib/received-vrm-sound';
import { showReceiveNotification } from '@/lib/notifications/receive';
import { useUserPreferences } from '@/lib/user-preferences';
import { formatNumber } from '@/lib/utils';

function formatBatchMessage(batch: IncomingVrcBatch): {
  title: string;
  description?: string;
} {
  const amount = formatNumber(batch.totalAmount, 4);
  const count = batch.events.length;

  if (count === 1) {
    return {
      title: `Received ${amount} VRC`,
      description: '',
    };
  }

  return {
    title: `Received ${amount} VRC`,
    description: `${count} transactions`,
  };
}

/** Shows toast + plays chime when incoming VRC is detected (if enabled). */
export function useIncomingVrcNotifications(): void {
  const enabled = useUserPreferences((s) => s.prefs.notify_on_vrc_received !== false);
  const { mobileOnly } = useWalletMode();

  useEffect(() => {
    if (!enabled) return;

    return subscribeIncomingVrc((batch) => {
      const { title, description } = formatBatchMessage(batch);
      void showReceiveNotification({
        title,
        description: description || undefined,
        native: mobileOnly,
      });
      void playReceivedVrmSound();
    });
  }, [enabled, mobileOnly]);
}
