//! HTTP status text helpers for browser JavaScript responses.

pub(crate) fn status_text(status: u16) -> &'static str {
    match status {
        200..=299 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        500..=599 => "Internal Server Error",
        _ => "",
    }
}
