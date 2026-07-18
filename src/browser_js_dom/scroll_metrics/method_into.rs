use super::*;

pub(super) fn install(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    handle: &DomHandle,
) {
    let into_view = handle.clone();
    let into_object = object.clone();
    object.borrow_mut().insert(
        "scrollIntoView".into(),
        native("scrollIntoView", None, move |args| {
            let handle = access::current_handle(&into_object, &into_view);
            into_view::run(&handle, args.first())?;
            Ok(JsValue::Undefined)
        }),
    );
    object.borrow_mut().insert("onscroll".into(), JsValue::Null);
    let handler = handle.clone();
    let handler_object = object.clone();
    object.borrow_mut().insert(
        "__set:onscroll".into(),
        native("set_onscroll", Some(1), move |args| {
            let handle = access::current_handle(&handler_object, &handler);
            handle.set_handler("onscroll", args.first().cloned().unwrap_or(JsValue::Null));
            Ok(JsValue::Undefined)
        }),
    );
}
