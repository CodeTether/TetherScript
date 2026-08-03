//! Streaming Server-Sent Events: response shape, buffering, and wire chunks.
//!
//! The sibling `sse` group already frames a single event correctly. What was
//! missing is everything around it: the response *shape* that marks a body as an
//! event stream, the batch buffering that makes a bounded stream expressible
//! against today's one-shot server, and the individual chunk primitives a future
//! streaming server will write incrementally.
//!
//! # Built-ins
//!
//! | Name | Result shape |
//! |---|---|
//! | `sse_stream_response(events)` | Result of a response map |
//! | `sse_stream_headers()` | map of headers |
//! | `sse_chunk(event)` | Result of str, exact wire bytes |
//! | `sse_keepalive()` | str, comment-only chunk |
//! | `sse_retry_frame(ms)` | Result of str |
//!
//! # The correctness traps, all handled here
//!
//! * **A blank line terminates an event.** Every chunk ends with one; a frame
//!   without it is an event that never fires while the socket looks healthy.
//! * **Multi-line `data` needs one `data:` line per line.** Otherwise the client
//!   truncates at the first newline with no error at all — see
//!   `ssestream_data`.
//! * **CRLF and lone CR are normalized to LF.** The wire format is LF-delimited,
//!   so a stray CR would end up inside the payload.
//! * **`id` must not contain NUL.** The client silently ignores such an id and
//!   replays from a stale position after reconnect, so it is rejected.
//! * **`cache-control: no-store`.** A cached event stream is served stale
//!   forever — see `ssestream_response`.
//! * **Comments (`:` lines) are the keepalive mechanism.** An idle stream with no
//!   traffic is a stream a proxy will buffer and eventually drop.
//!
//! # This module does not implement true streaming
//!
//! `http_serve` writes one response and closes. `ssestream_spec` specifies
//! exactly what the server must add; that change belongs to whoever owns
//! `src/http_server*.rs`, not to this group.
//!
//! # Examples
//!
//! ```tether
//! let tick = map()
//! tick.event = "tick"
//! tick.data = "1"
//! return sse_stream_response([tick])?
//! ```
//!
//! # Layout
//!
//! * `ssestream_install` — built-in registration
//! * `ssestream_args` — argument adapters
//! * `ssestream_batch` — list of events to one body
//! * `ssestream_chunk` — one complete wire chunk
//! * `ssestream_field` — single-line field validation
//! * `ssestream_data` — multi-line `data` encoding
//! * `ssestream_response` — response and header maps
//! * `ssestream_spec` — the required `http_serve` change, documented only

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

#[path = "ssestream_args.rs"]
mod ssestream_args;
#[path = "ssestream_batch.rs"]
mod ssestream_batch;
#[path = "ssestream_chunk.rs"]
mod ssestream_chunk;
#[path = "ssestream_data.rs"]
mod ssestream_data;
#[path = "ssestream_field.rs"]
mod ssestream_field;
#[path = "ssestream_install.rs"]
mod ssestream_install;
#[path = "ssestream_response.rs"]
mod ssestream_response;
#[path = "ssestream_spec.rs"]
mod ssestream_spec;

/// Register this group's built-ins.
///
/// # Arguments
///
/// * `env` — Global environment to define the bindings in.
///
/// # Returns
///
/// Nothing; five names are defined as immutable bindings.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    ssestream_install::install(env);
}
