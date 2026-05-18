//! Fetch argument parsing.

use super::super::{FetchRequest, JsValue};

pub(super) fn from_fetch_args(args: &[JsValue]) -> FetchRequest {
    let input = args.first().unwrap_or(&JsValue::Undefined);
    let init = args.get(1).unwrap_or(&JsValue::Undefined);
    let mut request = from_value(input);
    super::init::apply(&mut request, init);
    request
}

fn from_value(value: &JsValue) -> FetchRequest {
    match value {
        JsValue::Object(obj) => super::build::object(&obj.borrow(), value),
        _ => super::build::string(value.display()),
    }
}
