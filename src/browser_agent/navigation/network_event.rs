//! Session network events for document navigations.

use crate::browser_agent::navigation::request::DocumentRequest;
use crate::{browser_agent::page::BrowserPage, browser_session::NetworkEvent};

pub(super) fn push(
    page: &mut BrowserPage,
    request: &DocumentRequest,
    status: Option<u16>,
    route: &str,
) {
    page.session.network.push(NetworkEvent::with_route_result(
        request.method.clone(),
        request.url.clone(),
        status,
        Some(route.into()),
    ));
}
