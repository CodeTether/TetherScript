//! Reading the `Last-Event-ID` resume header off a request.
//!
//! When a stream drops — a lost network, or a server dropping an over-budget client
//! per [`super::backpressure`] — `EventSource` reconnects on its own and sends the
//! `id:` of the last event it dispatched in a `Last-Event-ID` request header. A
//! server that keeps a replay log can then resume exactly, so no event is lost
//! across a reconnect. That is the whole reason `id:` exists.
//!
//! HTTP header names are case-insensitive and clients are not consistent here: the
//! header appears as `Last-Event-ID`, `Last-Event-Id`, and `last-event-id` in the
//! wild. Every lookup below compares ASCII-case-insensitively.

use std::collections::HashMap;

/// The canonical lowercase header name.
pub const HEADER_NAME: &str = "last-event-id";

/// Whether `name` is the resume header.
///
/// # Arguments
///
/// * `name` — A header name, in any case.
///
/// # Returns
///
/// `true` for any casing of `last-event-id`.
///
/// # Examples
///
/// ```rust
/// use tetherscript::sse::last_event_id::is_header;
///
/// assert!(is_header("Last-Event-ID"));
/// assert!(is_header("last-event-id"));
/// assert!(!is_header("If-None-Match"));
/// ```
pub fn is_header(name: &str) -> bool {
    name.eq_ignore_ascii_case(HEADER_NAME)
}

/// Read the resume id from a header map.
///
/// # Arguments
///
/// * `headers` — Request headers. Keys may be in any case, so an already-lowercased
///   map such as the one `http_serve` builds works unchanged.
///
/// # Returns
///
/// `Some(id)` when the header is present with a non-blank value, trimmed of
/// surrounding whitespace. `None` when the header is absent **or** blank: an empty
/// `Last-Event-ID` means "no resume position", so collapsing the two spares every
/// caller the same check.
///
/// The value is otherwise verbatim and **not** validated — it came from the client
/// and is opaque. Treat it as untrusted: look it up in a replay log, never
/// interpolate it into a response, and re-validate it with
/// [`super::validate::id_line`] before echoing it back as an `id:`.
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use tetherscript::sse::last_event_id::from_map;
///
/// let mut headers = HashMap::new();
/// assert_eq!(from_map(&headers), None);
///
/// headers.insert("Last-Event-ID".to_string(), " 42 ".to_string());
/// assert_eq!(from_map(&headers), Some("42"));
///
/// headers.insert("Last-Event-ID".to_string(), "  ".to_string());
/// assert_eq!(from_map(&headers), None);
/// ```
pub fn from_map(headers: &HashMap<String, String>) -> Option<&str> {
    for (name, value) in headers {
        if is_header(name.as_str()) {
            return normalize(value.as_str());
        }
    }
    None
}

/// Read the resume id from header pairs kept in arrival order.
///
/// # Arguments
///
/// * `headers` — `(name, value)` pairs in any case. The first non-blank match wins;
///   a duplicated header is a malformed request, and later copies are ignored
///   rather than joined so a second header cannot override the first.
///
/// # Returns
///
/// `Some(id)` for the first non-blank match, else `None`.
///
/// # Examples
///
/// ```rust
/// use tetherscript::sse::last_event_id::from_pairs;
///
/// let headers = [("Accept", "text/event-stream"), ("last-event-id", "7")];
/// assert_eq!(from_pairs(&headers), Some("7"));
/// assert_eq!(from_pairs(&[("Accept", "*/*")]), None);
/// ```
pub fn from_pairs<'a>(headers: &'a [(&'a str, &'a str)]) -> Option<&'a str> {
    for (name, value) in headers {
        if is_header(*name) {
            return normalize(*value);
        }
    }
    None
}

/// Trim a raw header value, mapping blank to `None`.
///
/// # Arguments
///
/// * `value` — Raw header value.
///
/// # Returns
///
/// The trimmed value, or `None` when it is empty.
fn normalize(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed)
}
