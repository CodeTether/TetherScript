//! Event-count and wall-clock bounds on a stream.
//!
//! ## Why bounds are mandatory, not advisory
//!
//! `http_serve` is a single-threaded accept loop. While a stream is open the loop
//! is blocked inside the pump, so *every other client waits*. An unbounded
//! generator is therefore not "a long-lived subscription", it is an outage: one
//! `while true` handler and the server never serves another request, health check
//! included.
//!
//! Both a count and a duration cap exist because they fail differently. A count
//! cap alone lets a generator that sleeps a minute between events hold the loop
//! for an hour; a duration cap alone lets a tight loop emit millions of events
//! and saturate the client's buffer inside the window. Neither can be disabled —
//! a handler may lower a bound but the defaults apply when it says nothing.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::value::Value;

/// Default cap on events per stream when the handler does not choose one.
pub(crate) const DEFAULT_MAX_EVENTS: u32 = 1_000;

/// Default cap on stream lifetime in milliseconds.
pub(crate) const DEFAULT_MAX_DURATION_MS: u64 = 30_000;

/// Hard ceiling no handler may exceed, however large its `max_events`.
pub(crate) const EVENT_CEILING: u32 = 100_000;

/// Hard ceiling on stream lifetime: ten minutes.
pub(crate) const DURATION_CEILING_MS: u64 = 600_000;

/// The two caps applied to one stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Bounds {
    /// Maximum payloads to flush.
    pub(crate) max_events: u32,
    /// Maximum wall-clock lifetime.
    pub(crate) max_duration: Duration,
}

#[path = "http_stream_response_bounds_parse.rs"]
mod parse;

impl Bounds {
    /// Read `max_events` and `max_duration_ms` from a response map.
    ///
    /// # Arguments
    ///
    /// * `map` — Borrowed streaming-response map.
    ///
    /// # Returns
    ///
    /// The bounds, each clamped to its ceiling.
    ///
    /// # Errors
    ///
    /// Returns `Err` naming the key when either value is present but is not a
    /// positive integer. Zero is rejected: a stream that may emit nothing is a
    /// mistake, not a configuration.
    pub(crate) fn parse(map: &HashMap<String, Value>) -> Result<Self, String> {
        Ok(Bounds {
            max_events: parse::events(map)?,
            max_duration: Duration::from_millis(parse::duration_ms(map)?),
        })
    }

    /// Report whether either bound has been reached.
    ///
    /// # Arguments
    ///
    /// * `events` — Payloads already flushed.
    /// * `started` — When the stream began.
    ///
    /// # Returns
    ///
    /// `Some(reason)` when the stream must stop, `None` to continue. Checked
    /// *before* each generator call so a bound can never be overshot by one.
    pub(crate) fn exceeded(&self, events: u32, started: Instant) -> Option<super::StopReason> {
        if events >= self.max_events {
            return Some(super::StopReason::MaxEvents);
        }
        if started.elapsed() >= self.max_duration {
            return Some(super::StopReason::MaxDuration);
        }
        None
    }
}
