import { describe, expect, it, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';
import { rpcSendToAddress } from '@/lib/rpc/client';

describe('backend security gates', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it('passes totpCode to send_to_address invoke', async () => {
    vi.mocked(invoke).mockResolvedValueOnce('txid-abc');
    await rpcSendToAddress('verium', 'VRq98Nm2P6anLHPgnHdb6NnibJ6GoG3Jm9', 1.5, '', '123456');
    expect(invoke).toHaveBeenCalledWith(
      'send_to_address',
      expect.objectContaining({
        totpCode: '123456',
        walletPassphrase: null,
        extraConfirmed: true,
      })
    );
  });

  it('passes walletPassphrase when 2FA is disabled', async () => {
    vi.mocked(invoke).mockResolvedValueOnce('txid-abc');
    await rpcSendToAddress(
      'verium',
      'VRq98Nm2P6anLHPgnHdb6NnibJ6GoG3Jm9',
      1.5,
      '',
      undefined,
      'my-wallet-pass'
    );
    expect(invoke).toHaveBeenCalledWith(
      'send_to_address',
      expect.objectContaining({
        totpCode: null,
        walletPassphrase: 'my-wallet-pass',
      })
    );
  });

  it('surfaces backend send auth rejection to callers', async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error('Wallet passphrase required to send'));
    await expect(
      rpcSendToAddress('verium', 'VRq98Nm2P6anLHPgnHdb6NnibJ6GoG3Jm9', 1)
    ).rejects.toThrow(/Wallet passphrase required/);
  });
});
