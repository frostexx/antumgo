use dashmap::DashMap;
use std::{
    sync::{atomic::AtomicU64, Arc},
    time::{Duration, Instant},
};
use tokio::time::sleep;
use tracing::{info, warn};

pub struct NetworkProtection {
    connection_pool: Arc<DashMap<String, ConnectionInfo>>,
    active_connections: Arc<AtomicU64>,
    max_connections: u64,
    flood_detection: Arc<FloodDetector>,
}

#[derive(Debug, Clone)]
struct ConnectionInfo {
    created_at: Instant,
    last_used: Instant,
    request_count: u64,
    success_count: u64,
}

pub struct FloodDetector {
    request_timestamps: Arc<DashMap<String, Vec<Instant>>>,
    flood_threshold: usize,
    time_window: Duration,
}

impl FloodDetector {
    pub fn new() -> Self {
        Self {
            request_timestamps: Arc::new(DashMap::new()),
            flood_threshold: 100, // 100 requests per second indicates flooding
            time_window: Duration::from_secs(1),
        }
    }

    pub fn is_flooding(&self, endpoint: &str) -> bool {
        let mut entry = self.request_timestamps.entry(endpoint.to_string()).or_insert_with(Vec::new);
        let now = Instant::now();
        
        // Remove old timestamps
        entry.retain(|&timestamp| now.duration_since(timestamp) <= self.time_window);
        
        // Add current timestamp
        entry.push(now);
        
        entry.len() > self.flood_threshold
    }

    pub async fn wait_if_flooding(&self, endpoint: &str) -> bool {
        if self.is_flooding(endpoint) {
            warn!("Flood detected on endpoint: {}, implementing back-pressure", endpoint);
            sleep(Duration::from_millis(100)).await;
            true
        } else {
            false
        }
    }
}

impl NetworkProtection {
    pub fn new() -> Self {
        Self {
            connection_pool: Arc::new(DashMap::new()),
            active_connections: Arc::new(AtomicU64::new(0)),
            max_connections: 1000, // High connection limit
            flood_detection: Arc::new(FloodDetector::new()),
        }
    }

    pub async fn acquire_connection(&self, endpoint: &str) -> Result<String, String> {
        use std::sync::atomic::Ordering;

        // Check for flooding
        self.flood_detection.wait_if_flooding(endpoint).await;

        // Check connection limit
        if self.active_connections.load(Ordering::SeqCst) >= self.max_connections {
            self.cleanup_stale_connections().await;
            
            if self.active_connections.load(Ordering::SeqCst) >= self.max_connections {
                return Err("Maximum connections reached".to_string());
            }
        }

        let connection_id = uuid::Uuid::new_v4().to_string();
        let connection_info = ConnectionInfo {
            created_at: Instant::now(),
            last_used: Instant::now(),
            request_count: 0,
            success_count: 0,
        };

        self.connection_pool.insert(connection_id.clone(), connection_info);
        self.active_connections.fetch_add(1, Ordering::SeqCst);

        Ok(connection_id)
    }

    pub fn release_connection(&self, connection_id: &str) {
        use std::sync::atomic::Ordering;

        if self.connection_pool.remove(connection_id).is_some() {
            self.active_connections.fetch_sub(1, Ordering::SeqCst);
        }
    }

    pub fn update_connection_stats(&self, connection_id: &str, success: bool) {
        if let Some(mut connection) = self.connection_pool.get_mut(connection_id) {
            connection.last_used = Instant::now();
            connection.request_count += 1;
            if success {
                connection.success_count += 1;
            }
        }
    }

    async fn cleanup_stale_connections(&self) {
        use std::sync::atomic::Ordering;

        let stale_threshold = Duration::from_secs(30);
        let now = Instant::now();
        let mut removed_count = 0;

        self.connection_pool.retain(|_, connection| {
            let is_stale = now.duration_since(connection.last_used) > stale_threshold;
            if is_stale {
                removed_count += 1;
            }
            !is_stale
        });

        if removed_count > 0 {
            self.active_connections.fetch_sub(removed_count, Ordering::SeqCst);
            info!("Cleaned up {} stale connections", removed_count);
        }
    }

    pub async fn get_connection_stats(&self) -> (usize, f64) {
        let total_connections = self.connection_pool.len();
        let mut total_requests = 0u64;
        let mut total_successes = 0u64;

        for connection in self.connection_pool.iter() {
            total_requests += connection.request_count;
            total_successes += connection.success_count;
        }

        let success_rate = if total_requests > 0 {
            (total_successes as f64) / (total_requests as f64)
        } else {
            0.0
        };

        (total_connections, success_rate)
    }
}

impl Default for NetworkProtection {
    fn default() -> Self {
        Self::new()
    }
}