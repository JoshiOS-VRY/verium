use std::collections::HashMap;
use std::hash::Hash;

/// Minimum-interval guard for retryable actions (daemon spawn, chain repair),
/// keyed per coin. Replaces ad-hoc "last_spawn_at" / cooldown flags with one
/// explicit policy that the supervisor FSM consults before acting.
///
/// Time is supplied by the caller as a monotonic millisecond counter so the
/// guard is pure and unit-testable (the app passes `Instant`-derived millis).
#[derive(Debug, Default)]
pub struct RetryGuard<K: Eq + Hash + Copy> {
    last_attempt_ms: HashMap<K, u64>,
    interval_ms: u64,
}

impl<K: Eq + Hash + Copy> RetryGuard<K> {
    pub fn new(interval_ms: u64) -> Self {
        Self {
            last_attempt_ms: HashMap::new(),
            interval_ms,
        }
    }

    /// True when enough time has elapsed since the last attempt for `key`.
    pub fn allowed(&self, key: K, now_ms: u64) -> bool {
        match self.last_attempt_ms.get(&key) {
            Some(&last) => now_ms.saturating_sub(last) >= self.interval_ms,
            None => true,
        }
    }

    /// Record an attempt for `key`. Returns true if the attempt was allowed
    /// (and was recorded); false if it was throttled (and not recorded).
    pub fn try_mark(&mut self, key: K, now_ms: u64) -> bool {
        if self.allowed(key, now_ms) {
            self.last_attempt_ms.insert(key, now_ms);
            true
        } else {
            false
        }
    }

    /// Milliseconds remaining before `key` may retry (0 when allowed now).
    pub fn cooldown_remaining_ms(&self, key: K, now_ms: u64) -> u64 {
        match self.last_attempt_ms.get(&key) {
            Some(&last) => self.interval_ms.saturating_sub(now_ms.saturating_sub(last)),
            None => 0,
        }
    }

    pub fn reset(&mut self, key: K) {
        self.last_attempt_ms.remove(&key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_attempt_is_allowed() {
        let mut guard: RetryGuard<u8> = RetryGuard::new(5_000);
        assert!(guard.try_mark(1, 0));
    }

    #[test]
    fn throttles_within_interval() {
        let mut guard: RetryGuard<u8> = RetryGuard::new(5_000);
        assert!(guard.try_mark(1, 0));
        assert!(!guard.try_mark(1, 4_999));
        assert_eq!(guard.cooldown_remaining_ms(1, 4_999), 1);
    }

    #[test]
    fn allows_after_interval() {
        let mut guard: RetryGuard<u8> = RetryGuard::new(5_000);
        assert!(guard.try_mark(1, 0));
        assert!(guard.try_mark(1, 5_000));
    }

    #[test]
    fn keys_are_independent() {
        let mut guard: RetryGuard<u8> = RetryGuard::new(5_000);
        assert!(guard.try_mark(1, 0));
        assert!(guard.try_mark(2, 0));
    }

    #[test]
    fn reset_clears_cooldown() {
        let mut guard: RetryGuard<u8> = RetryGuard::new(5_000);
        guard.try_mark(1, 0);
        guard.reset(1);
        assert!(guard.allowed(1, 1));
    }
}
