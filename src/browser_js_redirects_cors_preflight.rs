//! CORS preflight execution.

use super::super::super::{
    record_network_event, request_host, FetchRequest, SharedBrowserJsRouteHandler,
};

pub(super) fn run(
    request: &FetchRequest,
    route_handler: &SharedBrowserJsRouteHandler,
) -> Result<(), String> {
    let requested_headers = super::simple::requested_headers(request);
    let mut preflight = build(request, &requested_headers);
    request_host::refresh_headers(&mut preflight);
    let parts = super::super::response::once(&preflight, route_handler)?;
    record_network_event(
        &preflight.method,
        &preflight.url,
        Some(parts.status),
        parts.route_result.clone(),
    );
    super::validate_preflight::response(request, &parts, &requested_headers)
}

fn build(request: &FetchRequest, requested_headers: &[String]) -> FetchRequest {
    let mut headers = vec![(
        "access-control-request-method".into(),
        request.method.clone(),
    )];
    if !requested_headers.is_empty() {
        headers.push((
            "access-control-request-headers".into(),
            requested_headers.join(", "),
        ));
    }
    FetchRequest {
        url: request.url.clone(),
        method: "OPTIONS".into(),
        headers,
        body: None,
        aborted: false,
        credentials: "omit".into(),
        mode: "cors".into(),
        initiator_url: request.initiator_url.clone(),
    }
}
