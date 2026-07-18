use super::super::*;

pub(super) fn install(object: &Rc<RefCell<HashMap<String, JsValue>>>, listeners: Listeners) {
    insert(object, listeners.clone(), "dispatchEvent", false);
    insert(object, listeners, "__tsDispatchOrientationChange", true);
}

fn insert(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    listeners: Listeners,
    name: &'static str,
    trusted: bool,
) {
    let target = object.clone();
    object.borrow_mut().insert(
        name.into(),
        native(
            &format!("screen.orientation.{name}"),
            Some(1),
            move |args| {
                super::run(
                    &target,
                    &listeners,
                    args.first().cloned().unwrap_or(JsValue::Undefined),
                    trusted,
                )
            },
        ),
    );
}
