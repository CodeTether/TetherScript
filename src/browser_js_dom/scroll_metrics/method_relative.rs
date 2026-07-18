use super::*;

pub(super) fn install(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    handle: &DomHandle,
) {
    let handle = handle.clone();
    let object_ref = object.clone();
    object.borrow_mut().insert(
        "scrollBy".into(),
        native("scrollBy", None, move |args| {
            let handle = access::current_handle(&object_ref, &handle);
            let current = apply::current(&handle);
            let delta = args::relative(args);
            apply::to(
                &handle,
                state::Position {
                    left: current.left.saturating_add(delta.left),
                    top: current.top.saturating_add(delta.top),
                },
            )?;
            Ok(JsValue::Undefined)
        }),
    );
}
