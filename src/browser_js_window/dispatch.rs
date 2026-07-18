//! Native window event dispatchers used by JavaScript bootstrap functions.

use super::*;

pub(super) fn install(window: &JsValue) {
    let JsValue::Object(obj) = window else {
        return;
    };
    install_orientation_sync(obj, window.clone());
    install_one(obj, "__tsDispatchScroll", "scroll", window.clone());
    install_one(obj, "__tsDispatchResize", "resize", window.clone());
    install_one(
        obj,
        "__tsDispatchOrientationChange",
        "orientationchange",
        window.clone(),
    );
}

fn install_orientation_sync(obj: &Rc<RefCell<HashMap<String, JsValue>>>, window: JsValue) {
    obj.borrow_mut().insert(
        "__tsSyncOrientation".into(),
        native("__tsSyncOrientation", Some(0), move |_| {
            viewport_host::screen::sync_orientation(&window).map(JsValue::Bool)
        }),
    );
}

fn install_one(
    obj: &Rc<RefCell<HashMap<String, JsValue>>>,
    name: &'static str,
    event_type: &'static str,
    window: JsValue,
) {
    obj.borrow_mut().insert(
        name.into(),
        native(name, Some(0), move |_| {
            viewport_host::visual_viewport::sync(&window);
            super::trusted_event::dispatch(&window, event_type)?;
            viewport_host::visual_viewport::dispatch(&window, event_type)?;
            Ok(JsValue::Undefined)
        }),
    );
}
