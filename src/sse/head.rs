//! The `text/event-stream` response head.
//!
//! ## Why the head is its own concern
//!
//! Three header choices are what make a stream a stream, and getting any one wrong
//! breaks it in a way that looks like a hung server rather than an error:
//!
//! * `Content-Type: text/event-stream` — anything else and the browser refuses to
//!   treat the body as an `EventSource`.
//! * `Cache-Control: no-store` — a cached event stream is replayed stale forever.
//! * `Connection: keep-alive` — the socket must stay open after the head.
//!
//! ## No `Content-Length`, ever
//!
//! The body has no end, so its length is unknowable at head time. Writing any
//! `Content-Length` makes the client stop reading at that byte and treat what
//! follows as a protocol error; writing `0` makes it see an empty response and
//! close. This module therefore emits no `Content-Length`, and [`render`] drops one
//! a caller supplies. Framing is by connection close (HTTP/1.1 with no
//! `Transfer-Encoding`) or by chunking chosen upstream.
//!
//! The head is also written **before** the first event, so an `EventSource` opens
//! immediately instead of waiting on data that may be minutes away.

/// Headers this module always writes, in wire order.
///
/// # Examples
///
/// ```rust
/// assert_eq!(tetherscript::sse::head::HEADERS[0].0, "Content-Type");
/// ```
pub const HEADERS: [(&str, &str); 3] = [
    ("Content-Type", "text/event-stream; charset=utf-8"),
    ("Cache-Control", "no-store"),
    ("Connection", "keep-alive"),
];

/// Header names a caller may not override, compared ASCII-case-insensitively.
///
/// `content-length` is here because it would truncate the stream; the rest because
/// this module owns them.
pub const RESERVED: [&str; 5] = [
    "content-type",
    "content-length",
    "cache-control",
    "connection",
    "transfer-encoding",
];

/// Render the `200 OK` event-stream head.
///
/// # Returns
///
/// The complete head including its terminating blank line, ready to write before
/// any event. Infallible.
///
/// # Examples
///
/// ```rust
/// let head = tetherscript::sse::head::ok();
/// assert!(head.starts_with("HTTP/1.1 200 OK\r\n"));
/// assert!(head.contains("Content-Type: text/event-stream"));
/// assert!(!head.to_ascii_lowercase().contains("content-length"));
/// assert!(head.ends_with("\r\n\r\n"));
/// ```
pub fn ok() -> String {
    render(200, "OK", &[])
}

/// Render the head for `status`, plus any caller headers.
///
/// # Arguments
///
/// * `status` — HTTP status code. Only `2xx` makes sense for a stream; nothing here
///   enforces that, since a caller may be replaying a recorded response.
/// * `reason` — Reason phrase for `status`.
/// * `extra` — Additional headers as `(name, value)`. Names in [`RESERVED`] are
///   dropped. Values are written verbatim, so a caller must not pass one
///   containing CR or LF.
///
/// # Returns
///
/// The complete head including its terminating blank line. Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::sse::head::render;
///
/// // A proxy hint is welcome; a Content-Length is discarded.
/// let head = render(200, "OK", &[("X-Accel-Buffering", "no"), ("Content-Length", "5")]);
/// assert!(head.contains("X-Accel-Buffering: no\r\n"));
/// assert!(!head.to_ascii_lowercase().contains("content-length"));
/// ```
pub fn render(status: u16, reason: &str, extra: &[(&str, &str)]) -> String {
    let mut head = format!("HTTP/1.1 {status} {reason}\r\n");
    for (name, value) in HEADERS {
        push(&mut head, name, value);
    }
    for (name, value) in extra {
        if RESERVED.contains(&name.to_ascii_lowercase().as_str()) {
            continue;
        }
        push(&mut head, name, value);
    }
    head.push_str("\r\n");
    head
}

/// Render the `200 OK` head with extra headers appended.
///
/// # Arguments
///
/// * `extra` — Additional headers; see [`render`].
///
/// # Returns
///
/// The head, blank line included.
///
/// # Examples
///
/// ```rust
/// let head = tetherscript::sse::head::render_with(&[("X-Request-Id", "abc")]);
/// assert!(head.contains("X-Request-Id: abc\r\n"));
/// ```
pub fn render_with(extra: &[(&str, &str)]) -> String {
    render(200, "OK", extra)
}

/// Append one CRLF-terminated header line.
fn push(head: &mut String, name: &str, value: &str) {
    head.push_str(name);
    head.push_str(": ");
    head.push_str(value);
    head.push_str("\r\n");
}
