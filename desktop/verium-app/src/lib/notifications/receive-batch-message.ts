import {
  formatReceiveNotificationDescription,
  type ReceiveNotificationEvent,
} from '@/lib/notifications/receive-details';
import { formatNumber } from '@/lib/utils';

export function formatReceiveBatchMessage(
  events: ReceiveNotificationEvent[],
  totalAmount: number,
  symbol: 'VRM' | 'VRC'
): { title: string; description?: string } {
  const amount = formatNumber(totalAmount, 4);
  const title = `Received ${amount} ${symbol}`;

  if (events.length === 1) {
    const description = formatReceiveNotificationDescription(events[0]!);
    return { title, description };
  }

  return {
    title,
    description: `${events.length} transactions`,
  };
}
