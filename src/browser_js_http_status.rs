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

/// Return the path/query portion used for deterministic status hints.
pub(crate) fn status_target(url: &str) -> &str {
    let after_scheme = url.split_once("://").map_or(url, |(_, rest)| rest);
    after_scheme
        .find('/')
        .map_or("", |index| &after_scheme[index..])
}

/// Return whether a synthetic URL path requests a not-found response.
pub(crate) fn is_not_found_target(url: &str) -> bool {
    let target = status_target(url);
    target.contains("404") || target.contains("not-found")
}

#[cfg(test)]
mod tests {
    #[test]
    fn synthetic_status_target_excludes_host_and_port() {
        assert_eq!(super::status_target("http://127.0.0.1:50042/api"), "/api");
        assert_eq!(super::status_target("https://app.test/500?q=1"), "/500?q=1");
    }
}
