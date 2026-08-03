//! Specification of the `http_serve` change required for *true* streaming.
//!
//! This module contains no code. It exists because the streaming built-ins in
//! the `ssestream` group can express a stream's bytes but cannot keep a socket
//! open, and the integrator who owns `src/http_server*.rs` needs an exact
//! statement of what to add. **Nothing in this group implements any of the
//! following.**
//!
//! # Why the current server cannot stream
//!
//! `http_server_connection::handle` runs a strict request/response cycle: parse a
//! request, invoke the handler once, call `http_response::write_response`, loop.
//! `write_response` computes `Content-Length` from the finished body and calls
//! `flush` exactly once. The handler has already returned by the time any byte is
//! written, so there is no point at which script code can contribute a second
//! event. A stream is therefore inexpressible today — not slow, not buffered:
//! structurally absent.
//!
//! # Required change 1 — a streaming response variant
//!
//! Recognize a response map carrying a `stream` key whose value is callable
//! (`Value::Fn`, `Value::VmFn`, or `Value::Native`) with arity 0. That callable is
//! the *producer*. Its contract:
//!
//! * returns a str — write those bytes as one chunk and flush,
//! * returns `nil` — the stream is complete; finish and close,
//! * returns an `Err` Result — log it and close; the body is already committed so
//!   no status change is possible.
//!
//! Keep `status` and `headers` handling identical to the existing path. A response
//! map with both `stream` and `body` is a caller bug and must be rejected by
//! name, not silently resolved by preferring one.
//!
//! # Required change 2 — framing without `Content-Length`
//!
//! The length is unknown when the head is written, so `Content-Length` must be
//! omitted and one of these used instead:
//!
//! * `Transfer-Encoding: chunked` — each producer chunk written as
//!   `{len:X}\r\n{bytes}\r\n`, terminated by `0\r\n\r\n`. Required to keep the
//!   connection reusable afterwards, and required for HTTP/1.1 correctness.
//! * `Connection: close` with unbounded framing — bytes written raw until the
//!   socket closes. Simpler, but it burns the connection and defeats the
//!   `keep-alive` header this group emits.
//!
//! Chunked is the correct choice. Note that `http_response::write_parts`
//! unconditionally emits `Content-Length` and unconditionally emits `Connection`,
//! and `append_header` explicitly *drops* caller-supplied `content-length`,
//! `content-type`, and `connection`. A streaming writer therefore cannot reuse
//! `write_parts`; it needs a sibling function.
//!
//! # Required change 3 — flush per event
//!
//! `stream.flush()` after every chunk. Without it the OS and `BufWriter` layers
//! coalesce events, and a one-event-per-second feed arrives as a burst every
//! several seconds. Flush-per-event is the entire difference between a stream and
//! a slow batch.
//!
//! # Required change 4 — liveness and shutdown
//!
//! * Clear or lengthen the read timeout for the connection's lifetime.
//!   `http_server::serve` sets a 2 ms read timeout for keep-alive probing; a
//!   long-lived stream must not be torn down by it.
//! * Treat `ErrorKind::BrokenPipe` and `ConnectionReset` from a chunk write as
//!   normal client disconnect: stop calling the producer and return `Ok(())`. A
//!   disconnected `EventSource` is expected, not an error to log per event.
//! * Bound the total connection lifetime or the producer call count, so a
//!   producer that never returns `nil` cannot pin a thread forever. `http_serve`
//!   is single-threaded per connection and serves connections sequentially, so
//!   one unbounded stream currently blocks the entire listener. Flag this to the
//!   human: concurrent streaming needs either a thread per stream or scheduler
//!   integration, and that is a design decision, not an implementation detail.
//!
//! # What the built-ins already give the server
//!
//! * `sse_chunk(event)` — exact bytes of one event, ready to write and flush.
//! * `sse_keepalive()` — the comment chunk to emit on an idle tick.
//! * `sse_retry_frame(ms)` — the client's reconnect delay, normally sent first.
//! * `sse_stream_headers()` — the header map to attach to the streaming response.
//!
//! So the server change is confined to transport: no framing logic needs to be
//! written in Rust inside the server, and none should be.
