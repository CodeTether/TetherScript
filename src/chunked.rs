//! HTTP/1.1 chunked transfer encoding.
//!
//! `http_serve` writes one complete response and closes, so a handler cannot hold a
//! connection open and stream. That is why SSE could not be ported despite `sse_event`
//! existing: the framing was never the gap, the response shape was. This is the codec half
//! of that gap; wiring it into the server accept loop is separate work.
//!
//! Implements the chunked framing of RFC 9112 §7.1, plus the response head a streaming
//! response needs.

// Points at `mod.rs` rather than a bare file: a `#[path]`-included parent resolves its own
// submodules against its own directory, so `codec.rs` beside the files it declares would
// look for them one level too high.
#[path = "chunked/codec/mod.rs"]
pub mod codec;
