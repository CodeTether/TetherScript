use super::*;

pub(super) fn install(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    handle: &DomHandle,
) {
    for name in ["scroll", "scrollTo"] {
        let handle = handle.clone();
        let object_ref = object.clone();
        object.borrow_mut().insert(
            name.into(),
            native(name, None, move |args| {
                let handle = access::current_handle(&object_ref, &handle);
                let next = args::absolute(args, apply::current(&handle));
                apply::to(&handle, next)?;
                Ok(JsValue::Undefined)
            }),
        );
    }
}
