//! Route-table request construction for document navigations.

use crate::browser_agent::navigation::request::DocumentRequest;
use crate::browser_agent::network::RouteRequest;
use crate::browser_agent::page::BrowserPage;

pub(super) fn request(page: &BrowserPage, request: &DocumentRequest) -> RouteRequest {
    RouteRequest::new(&request.method, &request.url)
        .with_headers(headers(page, request))
        .with_optional_body(request.body.clone())
        .with_security(page.request_security_metadata(&request.url))
}

fn headers(page: &BrowserPage, request: &DocumentRequest) -> Vec<(String, String)> {
    let mut headers = request.headers.clone();
    if headers
        .iter()
        .any(|(name, _)| name.eq_ignore_ascii_case("cookie"))
    {
        return headers;
    }
    let cookie = page
        .session
        .cookie_header_for_request(&request.url, &page.session.url);
    if !cookie.is_empty() {
        headers.push(("cookie".into(), cookie));
    }
    headers
}
