//! CORS enforcement for native routed requests.

#[path = "browser_js_redirects_cors_preflight.rs"]
mod preflight_host;
#[path = "browser_js_redirects_cors_simple.rs"]
mod simple;
#[path = "browser_js_redirects_cors_validate.rs"]
mod validate;
#[path = "browser_js_redirects_cors_validate_preflight.rs"]
mod validate_preflight;

use super::super::{request_host, FetchRequest, FetchResponseParts, SharedBrowserJsRouteHandler};

pub(super) fn preflight(
    request: &FetchRequest,
    route_handler: &SharedBrowserJsRouteHandler,
) -> Result<(), String> {
    if !applies(request)? {
        return Ok(());
    }
    if simple::needs_preflight(request) {
        preflight_host::run(request, route_handler)?;
    }
    Ok(())
}

pub(super) fn actual(request: &FetchRequest, parts: &FetchResponseParts) -> Result<(), String> {
    if applies(request)? {
        validate::response(request, &parts.headers)?;
    }
    Ok(())
}

fn applies(request: &FetchRequest) -> Result<bool, String> {
    if !request_host::is_cross_origin(request) {
        return Ok(false);
    }
    if request.mode == "same-origin" {
        return Err(format!(
            "CORS blocked same-origin mode request from {} to {}",
            request_host::origin(&request.initiator_url),
            request_host::origin(&request.url)
        ));
    }
    Ok(request.mode == "cors")
}
