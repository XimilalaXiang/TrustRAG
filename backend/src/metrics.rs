use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde::Serialize;

#[derive(Default)]
pub struct Metrics {
    pub requests_total: AtomicU64,
    pub errors_total: AtomicU64,
    pub latency_sum_ms: AtomicU64,
    pub latency_max_ms: AtomicU64,
}

impl Metrics {
    pub fn record_request(&self, latency_ms: u64, is_error: bool) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        self.latency_sum_ms.fetch_add(latency_ms, Ordering::Relaxed);
        self.latency_max_ms.fetch_max(latency_ms, Ordering::Relaxed);
        if is_error {
            self.errors_total.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        let total = self.requests_total.load(Ordering::Relaxed);
        let errors = self.errors_total.load(Ordering::Relaxed);
        let latency_sum = self.latency_sum_ms.load(Ordering::Relaxed);
        let latency_max = self.latency_max_ms.load(Ordering::Relaxed);

        MetricsSnapshot {
            requests_total: total,
            errors_total: errors,
            error_rate: if total > 0 {
                errors as f64 / total as f64
            } else {
                0.0
            },
            avg_latency_ms: if total > 0 {
                latency_sum as f64 / total as f64
            } else {
                0.0
            },
            max_latency_ms: latency_max,
        }
    }
}

#[derive(Serialize)]
pub struct MetricsSnapshot {
    pub requests_total: u64,
    pub errors_total: u64,
    pub error_rate: f64,
    pub avg_latency_ms: f64,
    pub max_latency_ms: u64,
}

pub type SharedMetrics = Arc<Metrics>;

pub fn create_metrics() -> SharedMetrics {
    Arc::new(Metrics::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_recording() {
        let m = create_metrics();
        m.record_request(100, false);
        m.record_request(200, false);
        m.record_request(50, true);

        let snap = m.snapshot();
        assert_eq!(snap.requests_total, 3);
        assert_eq!(snap.errors_total, 1);
        assert!((snap.error_rate - 1.0 / 3.0).abs() < 0.001);
        assert!((snap.avg_latency_ms - 350.0 / 3.0).abs() < 0.1);
        assert_eq!(snap.max_latency_ms, 200);
    }

    #[test]
    fn test_metrics_empty() {
        let m = create_metrics();
        let snap = m.snapshot();
        assert_eq!(snap.requests_total, 0);
        assert_eq!(snap.error_rate, 0.0);
        assert_eq!(snap.avg_latency_ms, 0.0);
    }

    #[test]
    fn test_metrics_concurrent_safety() {
        use std::sync::Arc;
        let m = Arc::new(Metrics::default());
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let m = m.clone();
                std::thread::spawn(move || {
                    for _ in 0..100 {
                        m.record_request(10, i % 3 == 0);
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        let snap = m.snapshot();
        assert_eq!(snap.requests_total, 1000);
    }

    #[test]
    fn test_metrics_snapshot_serialization() {
        let m = create_metrics();
        m.record_request(42, false);
        let snap = m.snapshot();
        let json = serde_json::to_string(&snap).unwrap();
        assert!(json.contains("\"requests_total\":1"));
        assert!(json.contains("\"avg_latency_ms\":42"));
    }
}
