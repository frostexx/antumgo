use std::time::{Duration, Instant};
use tracing::{debug, info};
use crate::models::types::OptimizedFeeConfig;

pub struct FeeOptimizer {
    config: OptimizedFeeConfig,
    network_congestion_factor: f64,
    last_fee_update: Instant,
    competitor_fees: Vec<u64>,
}

impl FeeOptimizer {
    pub fn new() -> Self {
        Self {
            config: OptimizedFeeConfig::default(),
            network_congestion_factor: 1.0,
            last_fee_update: Instant::now(),
            competitor_fees: vec![3200000, 9400000], // Known competitor fees
        }
    }

    pub async fn calculate_optimal_claiming_fee(&self) -> u64 {
        // For claiming, sponsor pays, so we can be more aggressive
        let base_fee = 500000; // 0.5 PI base for claiming
        let sponsor_multiplier = 2.0; // Since sponsor pays, we can afford higher fees
        
        let optimal_fee = (base_fee as f64 * sponsor_multiplier * self.network_congestion_factor) as u64;
        
        debug!("Calculated optimal claiming fee: {} PI", optimal_fee as f64 / 1_000_000.0);
        optimal_fee
    }

    pub async fn calculate_optimal_transfer_fee(&self, amount: u64) -> u64 {
        // Get the highest competitor fee and add premium
        let max_competitor_fee = self.competitor_fees.iter().max().copied().unwrap_or(3200000);
        
        // Calculate our premium fee (significantly higher to ensure priority)
        let premium_multiplier = 2.5; // 150% higher than highest competitor
        let base_premium_fee = (max_competitor_fee as f64 * premium_multiplier) as u64;
        
        // Add network congestion factor
        let congestion_adjusted_fee = (base_premium_fee as f64 * self.network_congestion_factor) as u64;
        
        // Add amount-based scaling for larger transfers
        let amount_factor = if amount > 100_000_000 { // > 100 PI
            1.5
        } else if amount > 10_000_000 { // > 10 PI
            1.2
        } else {
            1.0
        };
        
        let final_fee = (congestion_adjusted_fee as f64 * amount_factor) as u64;
        
        // Cap at maximum fee
        let capped_fee = std::cmp::min(final_fee, self.config.max_fee);
        
        info!("Calculated optimal transfer fee: {} PI (competitor max: {} PI)", 
              capped_fee as f64 / 1_000_000.0, 
              max_competitor_fee as f64 / 1_000_000.0);
        
        capped_fee
    }

    pub async fn update_network_congestion(&mut self) {
        // Simulate network congestion monitoring
        // In real implementation, this would analyze network metrics
        
        let current_time = Instant::now();
        if current_time.duration_since(self.last_fee_update) > Duration::from_secs(10) {
            // Simulate dynamic congestion factor
            let random_factor = fastrand::f64() * 0.5 + 0.75; // 0.75 to 1.25
            self.network_congestion_factor = random_factor;
            self.last_fee_update = current_time;
            
            debug!("Updated network congestion factor: {:.2}", self.network_congestion_factor);
        }
    }

    pub fn add_competitor_fee(&mut self, fee: u64) {
        self.competitor_fees.push(fee);
        
        // Keep only recent competitor fees (last 10)
        if self.competitor_fees.len() > 10 {
            self.competitor_fees.remove(0);
        }
        
        info!("Added competitor fee: {} PI, tracking {} competitor fees", 
              fee as f64 / 1_000_000.0, 
              self.competitor_fees.len());
    }

    pub fn get_priority_boost_fee(&self, urgency_level: UrgencyLevel) -> u64 {
        let base_boost = match urgency_level {
            UrgencyLevel::Low => 0,
            UrgencyLevel::Medium => 1_000_000,    // +1 PI
            UrgencyLevel::High => 3_000_000,      // +3 PI
            UrgencyLevel::Critical => 7_000_000,  // +7 PI
        };
        
        (base_boost as f64 * self.network_congestion_factor) as u64
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UrgencyLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for FeeOptimizer {
    fn default() -> Self {
        Self::new()
    }
}