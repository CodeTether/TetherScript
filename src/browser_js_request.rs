//! Fetch and XHR request construction.

#[path = "browser_js_request_build.rs"]
mod build;
#[path = "browser_js_request_fetch.rs"]
mod fetch;
#[path = "browser_js_request_headers.rs"]
mod headers;
#[path = "browser_js_request_init.rs"]
mod init;
#[path = "browser_js_request_object.rs"]
mod object;
#[path = "browser_js_request_xhr.rs"]
mod xhr;

use super::{FetchRequest, JsValue};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub(super) fn from_fetch_args(args: &[JsValue]) -> FetchRequest {
    fetch::from_fetch_args(args)
}

pub(super) fn resolve(
    request: &mut FetchRequest,
    location: &Rc<RefCell<HashMap<String, JsValue>>>,
) {
    headers::resolve(request, location);
}

pub(super) fn refresh_headers(request: &mut FetchRequest) {
    headers::refresh_headers(request);
}

pub(super) fn allows_credentials(request: &FetchRequest) -> bool {
    headers::allows_credentials(request)
}

pub(super) fn is_cross_origin(request: &FetchRequest) -> bool {
    headers::is_cross_origin(request)
}

pub(super) fn origin(url: &str) -> String {
    headers::origin(url)
}

pub(super) fn from_object_fields(obj: &HashMap<String, JsValue>) -> FetchRequest {
    object::from_object_fields(obj)
}

pub(super) fn from_xhr(
    xhr: &Rc<RefCell<HashMap<String, JsValue>>>,
    body: Option<String>,
    initiator_url: &str,
) -> FetchRequest {
    xhr::from_xhr(xhr, body, initiator_url)
}
