//! Route request construction for page subresources.

use crate::browser_agent::network::RouteRequest;
use crate::browser_agent::page::BrowserPage;

pub(super) fn request(page: &BrowserPage, url: &str) -> RouteRequest {
    let metadata = page.request_security_metadata(url);
    RouteRequest::new("GET", url)
        .with_headers(headers(page, url, metadata.same_origin))
        .with_security(metadata)
}

pub(super) fn blocked(request: &RouteRequest) -> Option<String> {
    let metadata = request.security.as_ref()?;
    if metadata.target_origin.is_opaque() || metadata.allowed_by_policy {
        return None;
    }
    Some(format!(
        "CORS blocked cross-origin request from {} to {}",
        metadata.request_origin.serialized(),
        metadata.target_origin.serialized()
    ))
}

fn headers(page: &BrowserPage, url: &str, same_origin: bool) -> Vec<(String, String)> {
    let mut headers = Vec::new();
    if !same_origin {
        headers.push(("origin".into(), page.current_origin().serialized()));
        return headers;
    }
    let cookie = page
        .session
        .cookie_header_for_request(url, &page.session.url);
    if !cookie.is_empty() {
        headers.push(("cookie".into(), cookie));
    }
    headers
}
