//! Route-table requests for document navigations.

use crate::browser_agent::navigation::request::DocumentRequest;
use crate::browser_agent::network::RouteAction;
use crate::browser_agent::page::BrowserPage;

pub(super) struct DocumentResponse {
    pub(super) status: u16,
    pub(super) headers: Vec<(String, String)>,
    pub(super) body: String,
}

pub(super) enum DocumentOutcome {
    Continue,
    Response(DocumentResponse),
}

pub(super) fn send(
    page: &mut BrowserPage,
    request: &DocumentRequest,
) -> Result<DocumentOutcome, String> {
    if page.network_routes.borrow().routes().is_empty() {
        return Ok(DocumentOutcome::Continue);
    }
    match route(page, request) {
        RouteAction::Fulfill(reply) => {
            super::network_event::push(page, request, Some(reply.status), "fulfill");
            Ok(DocumentOutcome::Response(DocumentResponse {
                status: reply.status,
                headers: reply.headers,
                body: reply.body,
            }))
        }
        RouteAction::Abort(reason) => {
            super::network_event::push(page, request, None, "abort");
            Err(reason)
        }
        RouteAction::Continue => {
            super::network_event::push(page, request, None, "continue");
            Ok(DocumentOutcome::Continue)
        }
    }
}

fn route(page: &mut BrowserPage, request: &DocumentRequest) -> RouteAction {
    let route_request = super::network_request_build::request(page, request);
    page.network_routes.borrow_mut().handle(route_request)
}
