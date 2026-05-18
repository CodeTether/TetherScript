//! Redirect request rewriting rules.

use super::super::{request_host, resolve_url, FetchRequest};

pub(super) fn is_redirect(status: u16) -> bool {
    matches!(status, 301 | 302 | 303 | 307 | 308)
}

pub(super) fn location(headers: &[(String, String)]) -> Option<String> {
    headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("location"))
        .map(|(_, value)| value.clone())
}

pub(super) fn request(previous: &FetchRequest, status: u16, location: &str) -> FetchRequest {
    let mut request = previous.clone();
    request.url = resolve_url(location, Some(&previous.url));
    if rewrites_to_get(status, &request.method) {
        request.method = "GET".into();
        request.body = None;
        remove_body_headers(&mut request.headers);
    }
    request_host::refresh_headers(&mut request);
    request
}

fn rewrites_to_get(status: u16, method: &str) -> bool {
    status == 303 || matches!(status, 301 | 302) && !matches!(method, "GET" | "HEAD")
}

fn remove_body_headers(headers: &mut Vec<(String, String)>) {
    headers.retain(|(name, _)| {
        !name.eq_ignore_ascii_case("content-type") && !name.eq_ignore_ascii_case("content-length")
    });
}
