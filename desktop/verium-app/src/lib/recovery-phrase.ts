/** Split a BIP39 phrase into normalized words (trim, collapse whitespace). */
export function recoveryPhraseWords(phrase: string): string[] {
  return phrase.trim().split(/\s+/).filter(Boolean);
}

/** Clipboard text: one numbered word per line (matches on-screen layout). */
export function formatNumberedRecoveryPhraseForCopy(phrase: string): string {
  return recoveryPhraseWords(phrase)
    .map((word, i) => `${i + 1}. ${word}`)
    .join('\n');
}

function normalizeRecoveryWordInput(value: string): string {
  return value.trim().toLowerCase();
}

/**
 * Check user answers against phrase word positions (0-based indices).
 * Mirrors `vericonomy_wallet_core::recovery::verify_words_at_indices`.
 */
export function verifyRecoveryWordsAtIndices(
  phrase: string,
  indices: number[],
  answers: string[]
): boolean {
  const words = recoveryPhraseWords(phrase);
  if (indices.length !== answers.length || indices.length === 0) {
    return false;
  }
  return indices.every((rawIdx, i) => {
    const idx = Number(rawIdx);
    if (!Number.isInteger(idx) || idx < 0 || idx >= words.length) {
      return false;
    }
    return normalizeRecoveryWordInput(words[idx]) === normalizeRecoveryWordInput(answers[i] ?? '');
  });
}
