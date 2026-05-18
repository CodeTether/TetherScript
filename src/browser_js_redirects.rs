//! Redirect following for native fetch and XMLHttpRequest.

#[path = "browser_js_redirects_next.rs"]
mod next;
#[path = "browser_js_redirects_response.rs"]
mod response;

use super::{
    network_cookie_host, record_network_event, FetchRequest, FetchResponseParts,
    SharedBrowserJsRouteHandler,
};

const MAX_REDIRECTS: usize = 20;

pub(super) fn fetch_response_parts(
    request: &FetchRequest,
    route_handler: &SharedBrowserJsRouteHandler,
) -> Result<FetchResponseParts, String> {
    let mut request = request.clone();
    for redirects in 0..=MAX_REDIRECTS {
        let parts = response::once(&request, route_handler)?;
        network_cookie_host::apply_response_headers(&request.url, &parts.headers);
        record_network_event(
            &request.method,
            &request.url,
            Some(parts.status),
            parts.route_result.clone(),
        );
        let Some(location) = next::location(&parts.headers) else {
            return Ok(parts);
        };
        if !next::is_redirect(parts.status) {
            return Ok(parts);
        }
        if redirects == MAX_REDIRECTS {
            return Err("fetch redirect limit exceeded".into());
        }
        request = next::request(&request, parts.status, &location);
    }
    Err("fetch redirect limit exceeded".into())
}
