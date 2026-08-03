//! Streaming (`text/event-stream`) responses for `http_serve`.
//!
//! ## Why this module exists
//!
//! `src/http_response.rs` writes one complete response — status line,
//! `Content-Length`, body — and then the connection handler returns and the
//! socket is dropped. Server-sent events need the opposite shape: the head goes
//! out first, then bytes trickle out over seconds or minutes while the handler is
//! still computing them. The framing built-ins (`sse_event`, `sse_comment`,
//! `sse_retry` in `src/web_builtins/sse.rs`) already produce correct bytes, but
//! there was nowhere to stream them *to*. This module is that missing server
//! shape; it is not a new built-in.
//!
//! ## How a streaming response is recognised
//!
//! A handler returns a map carrying the reserved key `stream`, whose value is a
//! **zero-argument callable**. See [`shape::is_stream`]. Both halves matter:
//!
//! * An ordinary response map (`status` / `headers` / `body`) never contains
//!   `stream`, so no existing handler changes meaning.
//! * Even a map that happened to contain a `stream` key still takes the ordinary
//!   path unless the value is a function. A function is exactly what the ordinary
//!   path cannot render — `http_response_extract` would stringify it into a
//!   nonsense body — so a callable under `stream` is unambiguous evidence of
//!   intent rather than a coincidence.
//!
//! A `stream` key holding a non-callable is a hard error naming the key, so a
//! typo is loud instead of silently degrading to a one-shot response.
//!
//! ## Single-threaded starvation — read this before shipping an SSE route
//!
//! `http_serve` runs **one** accept loop on **one** thread. While a stream is
//! open, that thread is inside this module and no other connection is served;
//! new clients sit in the kernel backlog. A stream is therefore not a background
//! activity, it is an exclusive lease on the whole server. That is why [`bounds`]
//! caps every stream by event count *and* wall-clock duration, and why those caps
//! are not optional: without them one runaway generator is a total outage,
//! health check included. A bounded stream is not a substitute for the missing
//! concurrency — it is a bound on the damage.
//!
//! ## Wiring
//!
//! The entry point is [`respond`]. It takes the reason-phrase lookup as a
//! function pointer so this module compiles unchanged wherever it is declared,
//! and so it never duplicates the table in `src/http_status.rs`.
//!
//! ## Module map
//!
//! | Module | Concern |
//! | --- | --- |
//! | [`shape`] | Recognition and validation of the streaming shape |
//! | [`fields`] | Per-key extraction and SSE header defaults |
//! | [`bounds`] | Event-count and duration caps |
//! | [`chunked`][chunk] | Transfer coding and chunk framing |
//! | [`head`] | Response-head bytes |
//! | [`pump`] | The produce-flush loop |
//! | [`write`] | Disconnect-aware socket writes |
//! | [`outcome`] | How a stream ended |

// Nothing calls [`respond`] until the integrator adds the hook shown in this
// task's report, so every item here is unreachable from the non-test build for
// exactly one commit. Delete this allow in the wiring commit rather than leaving
// it: an unused streaming module is a bug once the hook exists.
#![allow(dead_code)]

use std::io::Write;

use crate::value::{Runtime, Value};

#[path = "http_stream_response_bounds.rs"]
pub(crate) mod bounds;
#[path = "http_stream_response_chunk.rs"]
pub(crate) mod chunk;
#[path = "http_stream_response_fields.rs"]
pub(crate) mod fields;
#[path = "http_stream_response_head.rs"]
pub(crate) mod head;
#[path = "http_stream_response_outcome.rs"]
pub(crate) mod outcome;
#[path = "http_stream_response_pump.rs"]
pub(crate) mod pump;
#[path = "http_stream_response_shape.rs"]
pub(crate) mod shape;
#[path = "http_stream_response_write.rs"]
pub(crate) mod write;

pub(crate) use outcome::{Outcome, StopReason};
pub(crate) use shape::is_stream;

/// Serve a streaming response: write the head, then flush events as produced.
///
/// # Arguments
///
/// * `runtime` — Active engine, used to invoke the generator callable.
/// * `out` — The client socket. Written head-first and flushed per event.
/// * `resp` — The handler's return value. Must satisfy [`is_stream`].
/// * `reason` — Reason-phrase lookup, normally `http_status::reason_phrase`.
///
/// # Returns
///
/// An [`Outcome`] recording how many events were flushed and why the stream
/// stopped. A client disconnect, an exhausted generator, and a reached bound are
/// all *ordinary* outcomes, not errors: only a malformed response value or an
/// unexpected I/O failure produces `Err`.
///
/// # Errors
///
/// Returns `Err` when `resp` is not a valid streaming response (see
/// [`shape::parse`]) or when writing the head fails for a reason other than the
/// peer going away.
///
/// # Examples
///
/// The module is crate-private, so rustdoc cannot run this; the behaviour is
/// asserted over a real socket by `tests/http_sse_stream.rs`. This mirrors
/// `src/http_status.rs`, which documents its private helper the same way.
///
/// ```text
/// // handler returned: { status: 200, stream: fn() { .. }, max_events: 3 }
/// let outcome = respond(rt, &mut socket, &resp, reason_phrase)?;
/// assert_eq!(outcome.events, 3);
/// ```
pub(crate) fn respond<W: Write>(
    runtime: &mut dyn Runtime,
    out: &mut W,
    resp: &Value,
    reason: fn(u16) -> &'static str,
) -> Result<Outcome, String> {
    let spec = shape::parse(resp)?;
    if head::write_head(out, &spec, reason(spec.status))? == write::Flow::Closed {
        return Ok(Outcome::new(0, StopReason::Disconnected));
    }
    Ok(pump::run(runtime, out, &spec))
}

#[cfg(test)]
#[path = "http_stream_response_tests.rs"]
mod tests;
