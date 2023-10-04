use super::config::RetryStrategyConfig;
use core::time::Duration;
pub use tokio_retry::strategy::{ExponentialBackoff, FibonacciBackoff, FixedInterval};

#[derive(Debug, Clone)]
pub enum RetryStrategy {
    /// fixed interval strategy, given a duration in milliseconds.
    FixedInterval(FixedInterval),

    /// A retry strategy driven by exponential back-off.
    /// The power corresponds to the number of past attempts.
    ExponentialBackoff(ExponentialBackoff),

    /// A retry strategy driven by the fibonacci series.
    /// Each retry uses a delay which is the sum of the two previous delays.
    /// Depending on the problem at hand, a fibonacci retry strategy might perform better and lead
    /// to better throughput than the ExponentialBackoff strategy.
    FibonacciBackoff(FibonacciBackoff),
}

impl Iterator for RetryStrategy {
    type Item = Duration;
    fn next(&mut self) -> Option<Duration> {
        match self {
            Self::FixedInterval(strategy) => strategy.next(),
            Self::ExponentialBackoff(strategy) => strategy.next(),
            Self::FibonacciBackoff(strategy) => strategy.next(),
        }
    }
}

impl From<&RetryStrategyConfig> for RetryStrategy {
    fn from(config: &RetryStrategyConfig) -> Self {
        match config {
            RetryStrategyConfig::FixedInterval(duration) => {
                Self::FixedInterval(FixedInterval::new(*duration))
            },
            RetryStrategyConfig::ExponentialBackoff { base, factor, max_delay } => {
                let mut exponential_backoff = ExponentialBackoff::from_millis(*base);
                if let Some(factor) = factor.as_ref() {
                    exponential_backoff = exponential_backoff.factor(*factor);
                }
                if let Some(max_delay) = max_delay.as_ref() {
                    exponential_backoff = exponential_backoff.max_delay(*max_delay);
                }
                Self::ExponentialBackoff(exponential_backoff)
            },
            RetryStrategyConfig::FibonacciBackoff { initial, factor, max_delay } => {
                let mut fibonacci_backoff = FibonacciBackoff::from_millis(*initial);
                if let Some(factor) = factor.as_ref() {
                    fibonacci_backoff = fibonacci_backoff.factor(*factor);
                }
                if let Some(max_delay) = max_delay.as_ref() {
                    fibonacci_backoff = fibonacci_backoff.max_delay(*max_delay);
                }
                Self::FibonacciBackoff(fibonacci_backoff)
            },
        }
    }
}
