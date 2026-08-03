//! The event pump: invoke the generator, frame, flush, repeat.
//!
//! ## Flush after every event
//!
//! Each payload is written and flushed before the generator is asked for the
//! next one. This is not a performance choice to be revisited: an SSE client that
//! receives nothing until the connection closes is indistinguishable from a hung
//! server, and the whole point of the feature is incremental delivery. Buffering
//! here would defeat it entirely, and would also delay discovery of a client
//! disconnect, letting a generator spin against a dead socket.
//!
//! ## Termination
//!
//! Every exit is bounded. The loop stops on the first of: generator returns
//! `nil`, generator raises, a write reports the peer is gone, or a bound in
//! [`super::bounds`] is reached. There is no path that loops forever, because
//! `http_serve` is single-threaded and one that did would take the server down.

use std::io::Write;
use std::time::Instant;

use crate::value::Runtime;

use super::shape::StreamSpec;
use super::write::Flow;
use super::{Outcome, StopReason};

#[path = "http_stream_response_pump_emit.rs"]
mod emit;
#[path = "http_stream_response_pump_payload.rs"]
pub(crate) mod payload;

/// Stream events until the generator, the client, or a bound ends it.
///
/// # Arguments
///
/// * `runtime` — Active engine, used to call `spec.generator` with no arguments.
/// * `out` — Destination socket, flushed after every event.
/// * `spec` — The parsed streaming response.
///
/// # Returns
///
/// An [`Outcome`]. Never an `Err`: once the head is on the wire the status cannot
/// change, so every ending is reported as data for the caller to log.
pub(crate) fn run<W: Write>(runtime: &mut dyn Runtime, out: &mut W, spec: &StreamSpec) -> Outcome {
    let started = Instant::now();
    let mut events: u32 = 0;
    loop {
        if let Some(stop) = spec.bounds.exceeded(events, started) {
            return emit::finish(out, spec, events, stop);
        }
        match step(runtime, out, spec) {
            Step::Sent => events = events.saturating_add(1),
            Step::Stopped(stop) => return emit::finish(out, spec, events, stop),
            Step::Gone => return Outcome::new(events, StopReason::Disconnected),
        }
    }
}

/// Result of producing and flushing one event.
enum Step {
    /// A payload reached the client.
    Sent,
    /// The stream ended for the carried reason.
    Stopped(StopReason),
    /// The peer went away; no terminator is owed.
    Gone,
}

/// Ask the generator for one payload and flush it.
fn step<W: Write>(runtime: &mut dyn Runtime, out: &mut W, spec: &StreamSpec) -> Step {
    let produced = match runtime.invoke(&spec.generator, &[]) {
        Ok(value) => value,
        Err(error) => return Step::Stopped(StopReason::GeneratorError(error)),
    };
    let bytes = match payload::bytes(&produced) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return Step::Stopped(StopReason::Exhausted),
        Err(error) => return Step::Stopped(StopReason::GeneratorError(error)),
    };
    match emit::event(out, spec, &bytes) {
        Ok(Flow::Open) => Step::Sent,
        Ok(Flow::Closed) => Step::Gone,
        Err(error) => Step::Stopped(StopReason::GeneratorError(error)),
    }
}
