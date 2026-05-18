//! Single-hop response selection for redirected browser requests.

use super::super::{
    default_response_parts, live_response_parts, record_network_event, route_action_for,
    status_text, BrowserJsRouteAction, FetchRequest, FetchResponseParts,
    SharedBrowserJsRouteHandler,
};

pub(super) fn once(
    request: &FetchRequest,
    route_handler: &SharedBrowserJsRouteHandler,
) -> Result<FetchResponseParts, String> {
    match route_action_for(request, route_handler) {
        None | Some(BrowserJsRouteAction::PassThrough) => {
            live_response_parts(request).or_else(|_| Ok(default_response_parts(request, None)))
        }
        Some(BrowserJsRouteAction::Continue) => live_response_parts(request)
            .or_else(|_| Ok(default_response_parts(request, Some("continue")))),
        Some(BrowserJsRouteAction::Abort(reason)) => fail(request, "abort", reason),
        Some(BrowserJsRouteAction::Blocked(reason)) => fail(request, "blocked", reason),
        Some(BrowserJsRouteAction::Fulfill(response)) => Ok(FetchResponseParts::new(
            request,
            response.status,
            status_text(response.status).into(),
            response.headers,
            response.body,
            Some("fulfill".into()),
        )),
    }
}

fn fail<T>(request: &FetchRequest, result: &str, reason: String) -> Result<T, String> {
    record_network_event(&request.method, &request.url, None, Some(result.into()));
    Err(reason)
}
