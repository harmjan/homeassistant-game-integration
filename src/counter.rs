use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Count events in a period to calculate an FPS
pub struct EventRateCounter {
    max_duration: Duration,
    instants: VecDeque<Instant>,
}

impl EventRateCounter {
    pub fn new() -> EventRateCounter {
        let mut instants = VecDeque::new();
        instants.push_front(Instant::now());
        EventRateCounter {
            instants,
            max_duration: Duration::from_secs(60),
        }
    }

    fn total_duration(&self) -> Duration {
        *self.instants.front().unwrap() - *self.instants.back().unwrap()
    }

    pub fn feed(&mut self) {
        self.instants.push_front(Instant::now());
        // Remove instants that are too old but always keep 2
        while self.instants.len() > 2 && self.total_duration() >= self.max_duration {
            self.instants.pop_back().unwrap();
        }
    }

    pub fn get_event_rate_per_second(&self) -> f32 {
        let duration_per_frame = self.total_duration() / self.instants.len() as u32;

        1f32 / duration_per_frame.as_secs_f32()
    }
}
