
use std::time::{Duration, Instant};

pub struct Deadline { start: Instant, budget: Duration }
impl Deadline {
    pub fn new(ms: u64) -> Self { Self { start: Instant::now(), budget: Duration::from_millis(ms) } }
    pub fn remaining(&self) -> Duration {
        let elapsed = self.start.elapsed();
        if elapsed >= self.budget { Duration::from_millis(0) } else { self.budget - elapsed }
    }
    pub fn exceeded(&self) -> bool { self.start.elapsed() >= self.budget }
}
