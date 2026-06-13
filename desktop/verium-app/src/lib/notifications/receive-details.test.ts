import { describe, expect, it } from 'vitest';
import { formatReceiveBatchMessage } from '@/lib/notifications/receive-batch-message';
import { formatReceiveNotificationDescription } from '@/lib/notifications/receive-details';

describe('formatReceiveNotificationDescription', () => {
  it('includes address, time, block, and tx id', () => {
    const text = formatReceiveNotificationDescription({
      txid: 'a385758895f3390123456789abcdef0123456789abcdef0123456789ab',
      address: 'VM1ExampleAddressForTestingPurposesOnly123456',
      confirmations: 3,
      time: 1_718_280_000,
      blockheight: 2_847_301,
    });
    expect(text).toContain('VM1Example…');
    expect(text).toContain('Block 2,847,301');
    expect(text).toContain('Tx a385758895f3');
  });
});

describe('formatReceiveBatchMessage', () => {
  it('uses detail lines for a single receive', () => {
    const { title, description } = formatReceiveBatchMessage(
      [
        {
          txid: 'abc123def4567890abcdef1234567890abcdef1234567890abcdef123456',
          confirmations: 1,
          address: 'VM1abcExampleAddress123456789',
        },
      ],
      48.291,
      'VRM'
    );
    expect(title).toBe('Received 48.2910 VRM');
    expect(description).toContain('VM1abcExam');
    expect(description).toContain('Tx abc123');
  });
});
