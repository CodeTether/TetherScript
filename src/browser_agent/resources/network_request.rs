//! Route-table requests for page subresources.

use crate::browser_agent::network::{RouteAction, RouteRequest};
use crate::browser_agent::page::BrowserPage;

pub(super) struct ResourceResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

pub(super) fn send(page: &mut BrowserPage, url: &str) -> Result<Option<ResourceResponse>, String> {
    let request = super::request_build::request(page, url);
    if let Some(reason) = super::request_build::blocked(&request) {
        super::request_event::log(page, request, RouteAction::abort(reason.clone()));
        super::request_event::push(page, url, None, "blocked");
        return Err(reason);
    }
    if page.network_routes.borrow().routes().is_empty() {
        return Ok(None);
    }
    match route(page, request) {
        RouteAction::Fulfill(reply) => {
            super::request_event::push(page, url, Some(reply.status), "fulfill");
            Ok(Some(ResourceResponse {
                status: reply.status,
                headers: reply.headers,
                body: reply.body,
            }))
        }
        RouteAction::Abort(reason) => {
            super::request_event::push(page, url, None, "abort");
            Err(reason)
        }
        RouteAction::Continue => Ok(None),
    }
}

fn route(page: &mut BrowserPage, request: RouteRequest) -> RouteAction {
    page.network_routes.borrow_mut().handle(request)
}
