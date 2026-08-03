//! A runtime whose every generator call takes measurable wall-clock time.
//!
//! Split into its own file so the wall-clock bound can be tested without adding
//! a clock-injection seam to the pump: making the *generator* slow is what a
//! real slow stream does, so the test exercises the production code path.

use std::thread::sleep;
use std::time::Duration;

use crate::value::{Runtime, Value};

/// A runtime that sleeps `delay` then yields the same frame forever.
pub(super) struct SlowRuntime {
    /// Frame returned by every call.
    pub(super) frame: Value,
    /// Wall-clock cost of each call.
    pub(super) delay: Duration,
    /// How many times [`Runtime::invoke`] was called.
    pub(super) calls: usize,
}

impl SlowRuntime {
    /// Build a runtime that yields `frame` after sleeping `ms` per call.
    ///
    /// # Arguments
    ///
    /// * `frame` — Value returned by every invocation.
    /// * `ms` — Sleep per invocation, in milliseconds.
    ///
    /// # Returns
    ///
    /// The runtime, with `calls` at zero. Infallible.
    pub(super) fn new(frame: Value, ms: u64) -> Self {
        SlowRuntime {
            frame,
            delay: Duration::from_millis(ms),
            calls: 0,
        }
    }
}

impl Runtime for SlowRuntime {
    fn invoke(&mut self, _callee: &Value, _args: &[Value]) -> Result<Value, String> {
        self.calls += 1;
        sleep(self.delay);
        Ok(self.frame.clone())
    }
}
