//! Session and route logging for page subresources.

use crate::browser_agent::network::{RouteAction, RouteRequest};
use crate::browser_agent::page::BrowserPage;
use crate::browser_session::NetworkEvent;

pub(super) fn log(page: &mut BrowserPage, request: RouteRequest, action: RouteAction) {
    page.network_routes.borrow_mut().push_log(request, action);
}

pub(super) fn push(page: &mut BrowserPage, url: &str, status: Option<u16>, route: &str) {
    page.session.network.push(NetworkEvent::with_route_result(
        "GET",
        url,
        status,
        Some(route.into()),
    ));
}
