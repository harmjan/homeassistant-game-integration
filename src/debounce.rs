use std::time::{Duration, Instant};

/// The webhook only needs to get called once, only pass the true result on once for a stretch of
/// true signals
pub struct DebounceRisingEdge {
    period: Duration,
    last_rising: Instant,
}

impl DebounceRisingEdge {
    pub fn new() -> DebounceRisingEdge {
        DebounceRisingEdge {
            period: Duration::from_secs(5),
            last_rising: Instant::now(),
        }
    }

    pub fn feed(&mut self, event: bool) -> bool {
        let now = Instant::now();
        let result = event && (now - self.last_rising) >= self.period;
        if event {
            self.last_rising = now;
        }
        result
    }
}
