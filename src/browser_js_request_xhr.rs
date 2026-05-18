//! XMLHttpRequest request construction.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::super::{headers_from_value, FetchRequest, JsValue};

pub(super) fn from_xhr(
    xhr: &Rc<RefCell<HashMap<String, JsValue>>>,
    body: Option<String>,
    initiator_url: &str,
) -> FetchRequest {
    let mut request = {
        let xhr = xhr.borrow();
        FetchRequest {
            url: xhr.get("__url").map(JsValue::display).unwrap_or_default(),
            method: xhr
                .get("__method")
                .map(JsValue::display)
                .unwrap_or_else(|| "GET".into()),
            headers: xhr
                .get("__requestHeaders")
                .map(headers_from_value)
                .unwrap_or_default(),
            body,
            aborted: false,
            credentials: credentials(&xhr),
            mode: "cors".into(),
            initiator_url: initiator_url.into(),
        }
    };
    super::headers::refresh_headers(&mut request);
    request
}

fn credentials(xhr: &HashMap<String, JsValue>) -> String {
    if xhr.get("withCredentials").is_some_and(JsValue::truthy) {
        "include".into()
    } else {
        "same-origin".into()
    }
}
