//! Host extraction from URLs.

/// Extract the hostname from a URL string.
pub(super) fn host_of(url: &str) -> String {
    let at = match url.find("://") {
        Some(i) => i + 3,
        None => 0,
    };
    let rest = &url[at..];
    let end = rest.find('/').unwrap_or(rest.len());
    rest[..end]
        .split(':')
        .next()
        .unwrap_or(&rest[..end])
        .to_string()
}
