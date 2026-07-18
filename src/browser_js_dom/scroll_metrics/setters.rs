use super::*;

pub(super) fn install(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    handle: &DomHandle,
) {
    for name in ["scrollLeft", "scrollTop"] {
        let handle = handle.clone();
        let object_ref = object.clone();
        object.borrow_mut().insert(
            format!("__set:{name}"),
            native(name, Some(1), move |args| {
                let handle = access::current_handle(&object_ref, &handle);
                apply::axis(&handle, name, args::number(args.first()))?;
                Ok(JsValue::Undefined)
            }),
        );
    }
}
