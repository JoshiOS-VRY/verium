import { describe, expect, it } from 'vitest';
import {
  formatNumberedRecoveryPhraseForCopy,
  recoveryPhraseWords,
  verifyRecoveryWordsAtIndices,
} from '@/lib/recovery-phrase';

describe('recoveryPhraseWords', () => {
  it('normalizes surrounding whitespace', () => {
    expect(recoveryPhraseWords('  alpha   bravo  charlie  ')).toEqual([
      'alpha',
      'bravo',
      'charlie',
    ]);
  });
});

describe('formatNumberedRecoveryPhraseForCopy', () => {
  it('prefixes each word with its 1-based index', () => {
    expect(formatNumberedRecoveryPhraseForCopy('alpha bravo charlie')).toBe(
      '1. alpha\n2. bravo\n3. charlie'
    );
  });
});

describe('verifyRecoveryWordsAtIndices', () => {
  const phrase = 'alpha bravo charlie delta';

  it('accepts correct words case-insensitively', () => {
    expect(verifyRecoveryWordsAtIndices(phrase, [0, 2], ['ALPHA', 'Charlie'])).toBe(true);
  });

  it('rejects wrong words', () => {
    expect(verifyRecoveryWordsAtIndices(phrase, [0, 2], ['alpha', 'wrong'])).toBe(false);
  });

  it('rejects mismatched answer count', () => {
    expect(verifyRecoveryWordsAtIndices(phrase, [0], ['alpha', 'bravo'])).toBe(false);
  });
});
