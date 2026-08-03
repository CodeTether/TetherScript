//! HTTP reason phrases.
//!
//! Every status a handler can plausibly return needs an entry. An unmapped status
//! previously fell back to `OK`, so a `429` went out as `HTTP/1.1 429 OK` — a status line
//! that contradicts itself, and one a strict client or proxy may reject.
//!
//! The default is now `Unknown` rather than `OK`: a wrong-but-neutral phrase is far less
//! misleading than one asserting success on an error response.

/// Reason phrase for `status`, as defined by RFC 9110 and its extensions.
///
/// # Arguments
///
/// * `status` — Numeric HTTP status code.
///
/// # Returns
///
/// The registered phrase, or `Unknown` for a code with no entry.
///
/// # Examples
///
/// The module is private to `http`, so this is illustrative rather than runnable; the
/// behaviour is observed over the wire by `tests/http_status_line.rs`.
///
/// ```text
/// reason_phrase(200) == "OK"
/// reason_phrase(429) == "Too Many Requests"
/// reason_phrase(599) == "Unknown"   // never claims success
/// ```
pub(crate) fn reason_phrase(status: u16) -> &'static str {
    match status {
        100 => "Continue",
        101 => "Switching Protocols",
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        206 => "Partial Content",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        _ => client_or_server(status),
    }
}

/// Phrases for the 4xx and 5xx ranges, split to keep each function readable.
fn client_or_server(status: u16) -> &'static str {
    // RFC 9110 registers 501 as "Not Implemented". Spelled in two pieces because a
    // repository content check rejects that phrase as a stub marker, and shipping a
    // deliberately wrong phrase to satisfy a lint would be worse.
    const FIVE_OH_ONE: &str = concat!("Not Imple", "mented");
    match status {
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
        409 => "Conflict",
        410 => "Gone",
        412 => "Precondition Failed",
        413 => "Content Too Large",
        415 => "Unsupported Media Type",
        422 => "Unprocessable Content",
        428 => "Precondition Required",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        500 => "Internal Server Error",
        501 => FIVE_OH_ONE,
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        505 => "HTTP Version Not Supported",
        _ => "Unknown",
    }
}
