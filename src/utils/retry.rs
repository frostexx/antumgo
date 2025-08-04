use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct RetryStrategy {
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub max_attempts: usize,
    pub backoff_multiplier: f64,
}

impl RetryStrategy {
    pub fn exponential_backoff(
        initial_delay: Duration,
        max_delay: Duration,
        max_attempts: usize,
    ) -> Self {
        Self {
            initial_delay,
            max_delay,
            max_attempts,
            backoff_multiplier: 1.5,
        }
    }

    pub fn linear_backoff(
        initial_delay: Duration,
        max_delay: Duration,
        max_attempts: usize,
    ) -> Self {
        Self {
            initial_delay,
            max_delay,
            max_attempts,
            backoff_multiplier: 1.0,
        }
    }

    pub fn fixed_delay(delay: Duration, max_attempts: usize) -> Self {
        Self {
            initial_delay: delay,
            max_delay: delay,
            max_attempts,
            backoff_multiplier: 1.0,
        }
    }

    pub fn aggressive() -> Self {
        Self {
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            max_attempts: 100,
            backoff_multiplier: 1.1,
        }
    }
}

pub struct RetryConfig {
    strategy: RetryStrategy,
}

impl RetryConfig {
    pub fn new(strategy: RetryStrategy) -> Self {
        Self { strategy }
    }

    pub async fn execute_with_retry<F, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Result<T, E>,
        E: std::fmt::Display,
    {
        let mut attempt = 0;
        let mut delay = self.strategy.initial_delay;

        loop {
            attempt += 1;
            
            match operation() {
                Ok(result) => {
                    if attempt > 1 {
                        debug!("Operation succeeded on attempt {}", attempt);
                    }
                    return Ok(result);
                }
                Err(error) => {
                    if attempt >= self.strategy.max_attempts {
                        warn!("Operation failed after {} attempts: {}", attempt, error);
                        return Err(error);
                    }

                    debug!("Attempt {} failed: {}, retrying in {:?}", attempt, error, delay);
                    sleep(delay).await;

                    // Calculate next delay
                    if self.strategy.backoff_multiplier > 1.0 {
                        delay = std::cmp::min(
                            Duration::from_millis(
                                (delay.as_millis() as f64 * self.strategy.backoff_multiplier) as u64
                            ),
                            self.strategy.max_delay,
                        );
                    }
                }
            }
        }
    }
}