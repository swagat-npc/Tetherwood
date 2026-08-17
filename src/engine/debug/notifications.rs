use std::time::{Duration, Instant};

pub struct Notification {
    pub message: String,
    pub duration: Duration,
    pub start_time: Instant,
}

impl Notification {
    pub fn expired(&self) -> bool {
        self.start_time.elapsed() > self.duration
    }
}
