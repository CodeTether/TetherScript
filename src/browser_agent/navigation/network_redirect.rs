//! Redirect request rewriting for document navigations.

use crate::browser_agent::navigation::request::DocumentRequest;

pub(super) fn follow(mut request: DocumentRequest, status: u16, location: &str) -> DocumentRequest {
    request.url = super::url::resolve(&request.url, location);
    if rewrites_to_get(status, &request.method) {
        request.method = "GET".into();
        request.body = None;
        remove_body_headers(&mut request.headers);
    }
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
