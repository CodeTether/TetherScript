//! Opaque WebGL uniform-location objects.

use super::*;

pub(super) fn object(state: &mut State, program: u32, name: &str) -> JsValue {
    let key = (program, name.to_string());
    if let Some(location) = state.uniform_locations.get(&key) {
        return location.clone();
    }
    let location = JsValue::Object(Rc::new(RefCell::new(HashMap::from([
        ("__webgl_kind".into(), JsValue::String("uniform".into())),
        ("__webgl_program".into(), JsValue::Number(program as f64)),
        ("__webgl_name".into(), JsValue::String(name.into())),
    ]))));
    state.uniform_locations.insert(key, location.clone());
    location
}

pub(super) fn parse(state: &State, value: Option<&JsValue>) -> Option<(u32, String)> {
    let JsValue::Object(actual) = value? else {
        return None;
    };
    let key = {
        let object = actual.borrow();
        let program = number(object.get("__webgl_program"))?;
        let JsValue::String(name) = object.get("__webgl_name")? else {
            return None;
        };
        (program, name.clone())
    };
    match state.uniform_locations.get(&key) {
        Some(JsValue::Object(expected)) if Rc::ptr_eq(actual, expected) => Some(key),
        _ => None,
    }
}

fn number(value: Option<&JsValue>) -> Option<u32> {
    match value {
        Some(JsValue::Number(value)) if value.is_finite() => Some(*value as u32),
        _ => None,
    }
}
