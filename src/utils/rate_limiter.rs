use governor::{Quota, RateLimiter as GovernorRateLimiter};
use nonzero_ext::nonzero;
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

pub struct RateLimiter {
    claiming_limiter: Arc<GovernorRateLimiter<String, dashmap::DashMap<String, governor::InMemoryState>, governor::clock::DefaultClock>>,
    transfer_limiter: Arc<GovernorRateLimiter<String, dashmap::DashMap<String, governor::InMemoryState>, governor::clock::DefaultClock>>,
    api_limiter: Arc<GovernorRateLimiter<String, dashmap::DashMap<String, governor::InMemoryState>, governor::clock::DefaultClock>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        use governor::clock::DefaultClock;
        
        // Ultra-aggressive rate limits for maximum performance
        let claiming_quota = Quota::per_second(nonzero!(1000u32)); // 1000 claims per second
        let transfer_quota = Quota::per_second(nonzero!(500u32));  // 500 transfers per second
        let api_quota = Quota::per_second(nonzero!(2000u32));      // 2000 API calls per second

        Self {
            claiming_limiter: Arc::new(GovernorRateLimiter::dashmap_with_clock(claiming_quota, &DefaultClock::default())),
            transfer_limiter: Arc::new(GovernorRateLimiter::dashmap_with_clock(transfer_quota, &DefaultClock::default())),
            api_limiter: Arc::new(GovernorRateLimiter::dashmap_with_clock(api_quota, &DefaultClock::default())),
        }
    }

    pub async fn check_claiming_rate(&self, key: &str) -> bool {
        match self.claiming_limiter.check_key(&key.to_string()) {
            Ok(_) => true,
            Err(_) => {
                // Very short wait to avoid rate limit
                sleep(Duration::from_millis(1)).await;
                false
            }
        }
    }

    pub async fn check_transfer_rate(&self, key: &str) -> bool {
        match self.transfer_limiter.check_key(&key.to_string()) {
            Ok(_) => true,
            Err(_) => {
                sleep(Duration::from_millis(2)).await;
                false
            }
        }
    }

    pub async fn check_api_rate(&self, key: &str) -> bool {
        match self.api_limiter.check_key(&key.to_string()) {
            Ok(_) => true,
            Err(_) => {
                sleep(Duration::from_millis(1)).await;
                false
            }
        }
    }

    pub async fn wait_for_claiming_slot(&self, key: &str) {
        while !self.check_claiming_rate(key).await {
            sleep(Duration::from_millis(1)).await;
        }
    }

    pub async fn wait_for_transfer_slot(&self, key: &str) {
        while !self.check_transfer_rate(key).await {
            sleep(Duration::from_millis(1)).await;
        }
    }
}