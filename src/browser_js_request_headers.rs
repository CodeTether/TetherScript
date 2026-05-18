//! Browser request header finalization.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::browser_cookie;

use super::super::{location_href, network_cookie_host, resolve_url, FetchRequest, JsValue};

pub(super) fn resolve(
    request: &mut FetchRequest,
    location: &Rc<RefCell<HashMap<String, JsValue>>>,
) {
    request.initiator_url = location_href(location);
    request.url = resolve_url(&request.url, Some(&request.initiator_url));
    refresh_headers(request);
}

pub(super) fn refresh_headers(request: &mut FetchRequest) {
    request.headers.retain(|(name, _)| {
        !name.eq_ignore_ascii_case("cookie") && !name.eq_ignore_ascii_case("origin")
    });
    network_cookie_host::set_document_url(&request.initiator_url);
    if let Some(origin) = origin_header(request) {
        request.headers.push(("origin".into(), origin));
    }
    if allows_credentials(request) {
        network_cookie_host::append_request_header(
            &mut request.headers,
            &request.url,
            &request.initiator_url,
        );
    }
}

pub(super) fn allows_credentials(request: &FetchRequest) -> bool {
    match request.credentials.as_str() {
        "omit" => false,
        "include" => true,
        _ => !is_cross_origin(request),
    }
}

pub(super) fn is_cross_origin(request: &FetchRequest) -> bool {
    if request.url.starts_with("data:") {
        return false;
    }
    origin(&request.url) != origin(&request.initiator_url)
}

pub(super) fn origin(url: &str) -> String {
    browser_cookie::storage_origin(url)
}

fn origin_header(request: &FetchRequest) -> Option<String> {
    (request.mode == "cors" && is_cross_origin(request)).then(|| origin(&request.initiator_url))
}
