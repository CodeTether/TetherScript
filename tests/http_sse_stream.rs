//! Server-sent-events streaming, observed on a raw socket.
//!
//! ## Why raw bytes and not a client library
//!
//! Every property that matters here is a *byte* property. A convenience client
//! would normalise the head, buffer the body, and hide exactly the bugs this
//! feature can have: a `Content-Length` promising a length nobody knows, a
//! decimal chunk length, a missing terminator, or events that only arrive once
//! the socket closes. So these tests speak HTTP/1.1 by hand, read from a
//! `TcpStream`, and assert on bytes.
//!
//! ## What is asserted, and where
//!
//! | File | Property |
//! | --- | --- |
//! | `head.rs` | No `Content-Length`; framing is named; ordinary responses unaffected |
//! | `framing.rs` | `data:`/`\n\n` bytes exact, multi-line data, comment, retry |
//! | `incremental.rs` | The first event is readable while later ones are still produced |
//! | `chunked.rs` | Hex chunk lengths and the terminating `0\r\n\r\n` |
//! | `bound.rs` | A generator that never ends is stopped by `max_events` |
//! | `disconnect.rs` | Closing the client mid-stream leaves the server answering |
//!
//! ## Harness
//!
//! Modelled on `tests/http_status_line.rs`: spawn the real binary, wait for the
//! server's own bind announcement on stderr, retry on port collisions rather than
//! assuming a port is free. See `server.rs`, `socket.rs`, and `response.rs`.
//!
//! ## Requires the server-loop wiring
//!
//! `src/http_stream_response.rs` provides the streaming shape; the integrator
//! hooks it into `src/http_server_connection.rs` so a handler returning a
//! `stream` map reaches it. Until that hook lands these tests fail at the first
//! assertion, because `http_serve` still renders the map as an ordinary response.
//! That is the intended signal, not a flake.

#[path = "http_sse_stream/response.rs"]
mod response;
#[path = "http_sse_stream/script.rs"]
mod script;
#[path = "http_sse_stream/server.rs"]
mod server;
#[path = "http_sse_stream/socket.rs"]
mod socket;

#[path = "http_sse_stream/bound.rs"]
mod bound;
#[path = "http_sse_stream/chunked.rs"]
mod chunked;
#[path = "http_sse_stream/disconnect.rs"]
mod disconnect;
#[path = "http_sse_stream/framing.rs"]
mod framing;
#[path = "http_sse_stream/head.rs"]
mod head;
#[path = "http_sse_stream/incremental.rs"]
mod incremental;
