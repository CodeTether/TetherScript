//! Shared `ImageData` prototype and constructor identity.

use super::*;

thread_local! {
    static CURRENT: RefCell<Option<JsValue>> = const { RefCell::new(None) };
}

pub(super) fn constructor() -> JsValue {
    let prototype = JsValue::Object(Rc::new(RefCell::new(HashMap::new())));
    CURRENT.with(|current| *current.borrow_mut() = Some(prototype.clone()));
    JsValue::Native(Rc::new(
        NativeFunction::new("ImageData", None, super::create::construct)
            .with_property("prototype", prototype),
    ))
}

pub(super) fn attach(value: &JsValue) -> Result<(), String> {
    let prototype = CURRENT
        .with(|current| current.borrow().clone())
        .unwrap_or_else(|| JsValue::Object(Rc::new(RefCell::new(HashMap::new()))));
    js::set_host_property(value, "__proto__", prototype)
}
