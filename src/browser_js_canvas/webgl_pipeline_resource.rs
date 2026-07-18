//! Opaque JavaScript handles for WebGL resources.

use super::*;

pub(super) fn new_object(kind: &str, id: u32) -> JsValue {
    JsValue::Object(Rc::new(RefCell::new(HashMap::from([
        ("__webgl_kind".into(), JsValue::String(kind.into())),
        ("__webgl_id".into(), JsValue::Number(id as f64)),
    ]))))
}

pub(super) fn id(state: &State, value: Option<&JsValue>, kind: &str) -> Option<u32> {
    let JsValue::Object(actual) = value? else {
        return None;
    };
    let id = {
        let object = actual.borrow();
        let valid =
            matches!(object.get("__webgl_kind"), Some(JsValue::String(name)) if name == kind);
        match object.get("__webgl_id") {
            Some(JsValue::Number(id)) if valid && id.is_finite() && *id > 0.0 => *id as u32,
            _ => return None,
        }
    };
    match state.objects.get(&id) {
        Some(JsValue::Object(expected)) if Rc::ptr_eq(actual, expected) => Some(id),
        _ => None,
    }
}
