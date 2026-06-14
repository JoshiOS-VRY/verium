import { describe, expect, it } from 'vitest';
import { formatDisplayError } from './format-display-error';

describe('formatDisplayError', () => {
  it('maps standard RPC codes', () => {
    expect(formatDisplayError('rpc error -6: Insufficient funds')).toBe(
      'Not enough spendable balance for this amount and fee.'
    );
    expect(formatDisplayError('rpc error -14: wrong passphrase')).toBe(
      'Incorrect wallet passphrase.'
    );
  });

  it('maps electrum rate limits', () => {
    expect(formatDisplayError('electrum error -101: excessive resource')).toBe(
      'The light-wallet server is busy. Wait a moment and try again.'
    );
  });

  it('refines common wallet messages', () => {
    expect(formatDisplayError('rpc error -4: min relay fee not met')).toBe(
      'The network fee is too low. Raise the fee and try again.'
    );
    expect(formatDisplayError('Wallet passphrase is required to send.')).toBe(
      'Enter your wallet passphrase to send.'
    );
  });

  it('passes through readable daemon messages', () => {
    expect(formatDisplayError('rpc error -4: Fee amount not valid')).toBe('Fee amount not valid');
  });

  it('handles Error objects', () => {
    expect(formatDisplayError(new Error('rpc error -28: warming up'))).toBe(
      'The node is still starting up. Try again in a moment.'
    );
  });
});
