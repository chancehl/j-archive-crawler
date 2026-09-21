use std::time::Duration;

use rand::Rng;

/// Politeness delay applied between consecutive episode fetches.
///
/// Each wait is `base` plus a random amount in `0..=jitter`, so a long crawl
/// does not hit j-archive on a perfectly fixed cadence.
#[derive(Debug, Clone, Copy)]
pub struct CrawlDelay {
    base_ms: u64,
    jitter_ms: u64,
}

impl CrawlDelay {
    /// Creates a delay from millisecond values
    pub fn new(base_ms: u64, jitter_ms: u64) -> Self {
        CrawlDelay { base_ms, jitter_ms }
    }

    /// True when no waiting should happen at all
    pub fn is_zero(&self) -> bool {
        self.base_ms == 0 && self.jitter_ms == 0
    }

    /// Draws a single wait duration: base + random(0..=jitter)
    pub fn sample(&self) -> Duration {
        let extra = if self.jitter_ms == 0 {
            0
        } else {
            rand::thread_rng().gen_range(0..=self.jitter_ms)
        };

        Duration::from_millis(self.base_ms.saturating_add(extra))
    }
}

#[cfg(test)]
mod tests {
    use super::CrawlDelay;
    use std::time::Duration;

    #[test]
    fn zero_delay_is_zero() {
        assert!(CrawlDelay::new(0, 0).is_zero());
        assert!(!CrawlDelay::new(0, 1).is_zero());
        assert!(!CrawlDelay::new(1, 0).is_zero());
    }

    #[test]
    fn sample_without_jitter_is_exactly_base() {
        let delay = CrawlDelay::new(250, 0);

        for _ in 0..100 {
            assert_eq!(delay.sample(), Duration::from_millis(250));
        }
    }

    #[test]
    fn sample_stays_within_bounds() {
        let delay = CrawlDelay::new(1000, 500);

        for _ in 0..1000 {
            let sampled = delay.sample();

            assert!(sampled >= Duration::from_millis(1000));
            assert!(sampled <= Duration::from_millis(1500));
        }
    }

    #[test]
    fn sample_actually_varies() {
        let delay = CrawlDelay::new(0, 10_000);

        let first = delay.sample();
        let varied = (0..100).any(|_| delay.sample() != first);

        assert!(varied, "jitter produced the same value 100 times");
    }
}
