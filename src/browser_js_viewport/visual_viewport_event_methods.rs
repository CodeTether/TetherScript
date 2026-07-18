use super::super::*;

pub(super) fn install(event: &Rc<RefCell<HashMap<String, JsValue>>>) {
    let for_prevent = event.clone();
    event.borrow_mut().insert(
        "preventDefault".into(),
        native("Event.preventDefault", Some(0), move |_| {
            if for_prevent
                .borrow()
                .get("cancelable")
                .is_some_and(JsValue::truthy)
            {
                for_prevent
                    .borrow_mut()
                    .insert("defaultPrevented".into(), JsValue::Bool(true));
            }
            Ok(JsValue::Undefined)
        }),
    );
    let for_stop = event.clone();
    event.borrow_mut().insert(
        "stopImmediatePropagation".into(),
        native("Event.stopImmediatePropagation", Some(0), move |_| {
            for_stop
                .borrow_mut()
                .insert("__immediatePropagationStopped".into(), JsValue::Bool(true));
            Ok(JsValue::Undefined)
        }),
    );
}
