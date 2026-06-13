import { shortExplorerAddress, shortTxid } from '@/components/mobile/explorer/tx-detail-utils';
import { formatTransactionTime } from '@/lib/units';

/** Fields available when alerting on an incoming receive. */
export interface ReceiveNotificationEvent {
  txid: string;
  address?: string;
  confirmations: number;
  /** Unix seconds from wallet RPC / indexer. */
  time?: number;
  blockheight?: number;
}

function formatBlockLine(event: ReceiveNotificationEvent): string | null {
  if (event.blockheight != null && event.blockheight > 0) {
    return `Block ${event.blockheight.toLocaleString()}`;
  }
  if (event.confirmations === 0) {
    return 'Unconfirmed';
  }
  return null;
}

/** Secondary lines shown under the “Received X VRM/VRC” title (toast + push-style copy). */
export function formatReceiveNotificationDescription(event: ReceiveNotificationEvent): string {
  const lines: string[] = [];

  if (event.address?.trim()) {
    lines.push(shortExplorerAddress(event.address.trim()));
  }

  const time = formatTransactionTime(event.time);
  const block = formatBlockLine(event);
  const meta = [time !== '—' ? time : null, block].filter(Boolean);
  if (meta.length > 0) {
    lines.push(meta.join(' · '));
  }

  lines.push(`Tx ${shortTxid(event.txid)}`);

  return lines.join('\n');
}
