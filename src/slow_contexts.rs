//! Command contexts whose native completion is too slow to run on every key.
//!
//! The shell reports how long each gather took. A context that is slow twice
//! in a row stops getting automatic popups; one fast gather, which Tab still
//! triggers, makes it automatic again.

/// Slow gathers in a row before a context stops being automatic. One slow
/// gather is often a cold cache.
const STRIKES: u8 = 2;
const MAX_CONTEXTS: usize = 64;

#[derive(Default)]
pub struct SlowContexts {
    /// Least recently recorded first.
    entries: Vec<(String, u8)>,
}

impl SlowContexts {
    /// The words before the one being completed. `None` while the command word
    /// itself is being typed, which has no context to remember.
    fn key(command_context: &str) -> Option<&str> {
        let end = command_context.rfind(' ')?;
        Some(&command_context[..=end])
    }

    /// Record one gather. A threshold of zero disables the policy.
    pub fn record(&mut self, command_context: Option<&str>, gather_ms: u32, threshold_ms: u32) {
        let Some(key) = command_context.and_then(Self::key) else {
            return;
        };
        let previous = self
            .entries
            .iter()
            .position(|(entry, _)| entry == key)
            .map(|index| self.entries.remove(index).1);
        if threshold_ms == 0 || gather_ms < threshold_ms {
            return;
        }
        if self.entries.len() == MAX_CONTEXTS {
            self.entries.remove(0);
        }
        let strikes = previous.unwrap_or(0).saturating_add(1).min(STRIKES);
        self.entries.push((key.to_string(), strikes));
    }

    pub fn is_slow(&self, command_context: Option<&str>, threshold_ms: u32) -> bool {
        threshold_ms != 0
            && command_context.and_then(Self::key).is_some_and(|key| {
                self.entries
                    .iter()
                    .any(|(entry, strikes)| entry == key && *strikes >= STRIKES)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const THRESHOLD: u32 = 300;

    #[test]
    fn a_context_is_slow_after_two_slow_gathers_in_a_row() {
        let mut contexts = SlowContexts::default();
        contexts.record(Some("yay -S "), 2500, THRESHOLD);
        assert!(!contexts.is_slow(Some("yay -S a"), THRESHOLD));
        contexts.record(Some("yay -S a"), 400, THRESHOLD);
        assert!(contexts.is_slow(Some("yay -S "), THRESHOLD));
        assert!(contexts.is_slow(Some("yay -S zsh-"), THRESHOLD));
        assert!(!contexts.is_slow(Some("yay -R "), THRESHOLD));
    }

    #[test]
    fn one_fast_gather_makes_the_context_automatic_again() {
        let mut contexts = SlowContexts::default();
        contexts.record(Some("git "), 400, THRESHOLD);
        contexts.record(Some("git "), 299, THRESHOLD);
        contexts.record(Some("git "), 400, THRESHOLD);
        assert!(!contexts.is_slow(Some("git "), THRESHOLD));
        contexts.record(Some("git "), 300, THRESHOLD);
        assert!(contexts.is_slow(Some("git "), THRESHOLD));
        contexts.record(Some("git c"), 10, THRESHOLD);
        assert!(!contexts.is_slow(Some("git "), THRESHOLD));
    }

    #[test]
    fn the_command_word_and_a_zero_threshold_are_never_slow() {
        let mut contexts = SlowContexts::default();
        for _ in 0..3 {
            contexts.record(Some("yay"), 5000, THRESHOLD);
            contexts.record(None, 5000, THRESHOLD);
            contexts.record(Some("yay -S "), 5000, 0);
        }
        assert!(!contexts.is_slow(Some("yay"), THRESHOLD));
        assert!(!contexts.is_slow(None, THRESHOLD));
        assert!(!contexts.is_slow(Some("yay -S "), 0));
        assert!(!contexts.is_slow(Some("yay -S "), THRESHOLD));
    }

    #[test]
    fn the_least_recently_recorded_context_is_forgotten_first() {
        let mut contexts = SlowContexts::default();
        for index in 0..=MAX_CONTEXTS {
            for _ in 0..STRIKES {
                contexts.record(Some(&format!("command{index} ")), 400, THRESHOLD);
            }
        }
        assert!(!contexts.is_slow(Some("command0 "), THRESHOLD));
        assert!(contexts.is_slow(Some("command1 "), THRESHOLD));
        assert_eq!(contexts.entries.len(), MAX_CONTEXTS);
    }
}
