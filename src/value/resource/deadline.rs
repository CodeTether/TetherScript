//! Monotonic per-resource deadlines.

use std::time::{Duration, Instant};

#[derive(Default)]
pub(super) struct Deadline(Option<Instant>);

impl Deadline {
    pub(super) fn set_after(&mut self, duration: Duration) {
        self.0 = Instant::now().checked_add(duration);
    }

    pub(super) fn clear(&mut self) {
        self.0 = None;
    }

    pub(super) fn instant(&self) -> Option<Instant> {
        self.0
    }

    pub(super) fn expired(&self) -> bool {
        self.0.is_some_and(|deadline| Instant::now() >= deadline)
    }

    pub(super) fn remaining_ms(&self) -> Option<u64> {
        self.0.map(|deadline| {
            deadline
                .saturating_duration_since(Instant::now())
                .as_millis()
                .min(u64::MAX as u128) as u64
        })
    }
}
