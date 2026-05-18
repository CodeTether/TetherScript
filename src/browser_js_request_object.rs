//! Request object cloning.

use std::collections::HashMap;

use super::super::{FetchRequest, JsValue};

pub(super) fn from_object_fields(obj: &HashMap<String, JsValue>) -> FetchRequest {
    super::build::object(obj, &JsValue::Undefined)
}
