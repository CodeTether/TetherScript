use super::super::*;

const LOCK: &str = "__tsOrientationLock";

pub(super) fn follows_viewport(object: &Rc<RefCell<HashMap<String, JsValue>>>) -> bool {
    matches!(object.borrow().get(LOCK), None | Some(JsValue::Null))
        || matches!(object.borrow().get(LOCK), Some(JsValue::String(kind)) if kind == "any")
}

pub(super) fn set_lock(object: &Rc<RefCell<HashMap<String, JsValue>>>, kind: Option<&str>) {
    let value = kind.map_or(JsValue::Null, |kind| JsValue::String(kind.into()));
    object.borrow_mut().insert(LOCK.into(), value);
}
