/** Bounded in-memory txid set for notification watchers. */

const DEFAULT_MAX = 2_000;

export function capSeenTxids(set: Set<string>, max = DEFAULT_MAX): void {
  if (set.size <= max) return;
  const excess = set.size - max;
  const iter = set.values();
  for (let i = 0; i < excess; i++) {
    const next = iter.next();
    if (next.done) break;
    set.delete(next.value);
  }
}

export function addSeenTxid(set: Set<string>, txid: string, max = DEFAULT_MAX): void {
  set.add(txid);
  capSeenTxids(set, max);
}
