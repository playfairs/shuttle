use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone)]
pub struct ProgressTracker {
    bytes_transferred: Arc<AtomicU64>,
    total_bytes: u64,
    start_time: Instant,
}

impl ProgressTracker {
    pub fn new(total_bytes: u64) -> Self {
        ProgressTracker {
            bytes_transferred: Arc::new(AtomicU64::new(0)),
            total_bytes,
            start_time: Instant::now(),
        }
    }

    pub fn update(&self, bytes: u64) {
        self.bytes_transferred.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn current(&self) -> u64 {
        self.bytes_transferred.load(Ordering::Relaxed)
    }

    pub fn progress_percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.current() as f64 / self.total_bytes as f64) * 100.0
        }
    }

    pub fn elapsed_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}
