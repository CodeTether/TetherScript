//! # Server-sent events streaming transport
//!
//! [`EventStream`] is an outbound byte buffer that knows how to frame
//! `text/event-stream` events into itself. It owns **no socket**: a caller writes
//! [`EventStream::as_bytes`] (or drains with [`EventStream::take`]) through
//! whatever transport it already holds. That keeps every rule in this module
//! testable without binding a port.
//!
//! ## Why this module exists
//!
//! `http_serve` writes one complete response and closes, so there is nowhere for
//! an event stream to live. This module is the *transport shape* an incremental
//! writer needs: the response head, the frame encoder, the resume header, the
//! keepalive clock policy, and the memory bound. Wiring it into the server is a
//! separate concern and deliberately absent here — no built-in is registered and
//! no server file is touched.
//!
//! Wire framing for a *single* frame already exists as the `sse_event` /
//! `sse_comment` / `sse_retry` script built-ins in `crate::web_builtins`. This
//! module is the Rust-side transport those built-ins have no way to reach: buffer
//! ownership, the head, resume, keepalive timing, and backpressure.
//!
//! ## What lives where
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`event`] | The [`Event`] record and its fixed field order |
//! | [`fields`] | Infallible field encoders: `data`, `retry`, comment |
//! | [`validate`] | Fallible single-line encoders: `event`, `id` |
//! | [`buffer`] | Buffer ownership: append, inspect, drain |
//! | [`send`] | Framing helpers that append onto an [`EventStream`] |
//! | [`head`] | The `text/event-stream` response head |
//! | [`last_event_id`] | Reading `Last-Event-ID` off a request |
//! | [`keepalive`] | Clock-free "is a comment line due?" decision |
//! | [`backpressure`] | Buffer bound and the drop decision |
//! | [`error`] | [`SseError`], the one rejection type |
//!
//! ## The three rules that break clients silently
//!
//! 1. **Every event ends with a blank line.** That blank line is what makes the
//!    client dispatch. Omit it and the client buffers forever and shows nothing,
//!    with no error to explain it. [`Event::render`] always appends it.
//! 2. **A newline in a payload becomes another `data:` line.** A raw newline ends
//!    the field, silently truncating the event. CR and CRLF get the same
//!    treatment, and a lone CR does not produce a spurious extra event.
//! 3. **Never send `Content-Length`.** The body has no end; a length makes the
//!    client stop reading there. See [`head`].
//!
//! ## Wiring note for the server agent
//!
//! This file *is* the module root; there is no `mod.rs`. Expose it from `lib.rs`
//! with one line:
//!
//! ```rust,ignore
//! #[path = "sse/stream.rs"]
//! pub mod sse;
//! ```
//!
//! ## Quick start
//!
//! ```rust
//! use tetherscript::sse::{head, keepalive, Event, EventStream};
//!
//! // 1. Write the head once, before any event.
//! let response_head = head::ok();
//! assert!(response_head.contains("text/event-stream"));
//! assert!(!response_head.to_ascii_lowercase().contains("content-length"));
//!
//! // 2. Frame events into the stream, then drain and write.
//! let mut stream = EventStream::new();
//! stream.send(&Event::data("tick").name("clock").id("1")).unwrap();
//! assert_eq!(stream.take(), b"event: clock\nid: 1\ndata: tick\n\n".to_vec());
//!
//! // 3. Keep the socket warm while idle, and drop a client that stops reading.
//! if keepalive::is_due(15_000, 0, keepalive::DEFAULT_INTERVAL_MS) {
//!     stream.send_keepalive();
//! }
//! assert!(!stream.should_drop());
//! ```

#[path = "backpressure.rs"]
pub mod backpressure;
#[path = "buffer.rs"]
pub mod buffer;
#[path = "error.rs"]
pub mod error;
#[path = "event.rs"]
pub mod event;
#[path = "fields.rs"]
pub mod fields;
#[path = "head.rs"]
pub mod head;
#[path = "keepalive.rs"]
pub mod keepalive;
#[path = "last_event_id.rs"]
pub mod last_event_id;
#[path = "send.rs"]
pub mod send;
#[path = "validate.rs"]
pub mod validate;

pub use backpressure::DEFAULT_BOUND;
pub use error::SseError;
pub use event::Event;

/// An outbound `text/event-stream` byte buffer.
///
/// Frames are appended by the `send_*` methods in [`send`], inspected or drained
/// by the methods in [`buffer`], and bounded by [`backpressure`].
///
/// # Examples
///
/// ```rust
/// use tetherscript::sse::EventStream;
///
/// let mut stream = EventStream::with_bound(64);
/// assert_eq!(stream.bound(), 64);
/// stream.send_keepalive();
/// assert_eq!(stream.as_bytes(), b": ping\n");
/// ```
#[derive(Debug, Clone)]
pub struct EventStream {
    /// Bytes framed but not yet handed to the transport.
    buf: Vec<u8>,
    /// Ceiling on `buf` before the connection must be dropped.
    bound: usize,
}

impl EventStream {
    /// Create a stream bounded by [`DEFAULT_BOUND`] (64 KiB).
    ///
    /// # Returns
    ///
    /// An empty stream. Nothing is allocated until the first frame.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::sse::{EventStream, DEFAULT_BOUND};
    ///
    /// let stream = EventStream::new();
    /// assert_eq!(stream.buffered(), 0);
    /// assert_eq!(stream.bound(), DEFAULT_BOUND);
    /// ```
    pub fn new() -> Self {
        Self::with_bound(DEFAULT_BOUND)
    }

    /// Create a stream with an explicit buffer bound.
    ///
    /// # Arguments
    ///
    /// * `bound` — Ceiling in bytes on unwritten frames. A `bound` of `0` reports
    ///   [`EventStream::should_drop`] on the very first check, which is useful for
    ///   exercising the shutdown path and useless in production.
    ///
    /// # Returns
    ///
    /// An empty stream carrying `bound`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::sse::EventStream;
    ///
    /// assert!(!EventStream::with_bound(8).should_drop());
    /// assert!(EventStream::with_bound(0).should_drop());
    /// ```
    pub fn with_bound(bound: usize) -> Self {
        Self {
            buf: Vec::new(),
            bound,
        }
    }
}

impl Default for EventStream {
    fn default() -> Self {
        Self::new()
    }
}
