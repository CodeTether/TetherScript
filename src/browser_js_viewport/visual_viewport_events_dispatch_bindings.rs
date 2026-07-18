use super::super::*;

pub(super) fn install(object: &Rc<RefCell<HashMap<String, JsValue>>>, registry: model::Registry) {
    insert(object, registry.clone(), "dispatchEvent", false);
    insert(object, registry, "__tsDispatchViewportEvent", true);
}

fn insert(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    registry: model::Registry,
    name: &'static str,
    trusted: bool,
) {
    let target = object.clone();
    object.borrow_mut().insert(
        name.into(),
        native(&format!("visualViewport.{name}"), Some(1), move |args| {
            super::run(
                &target,
                &registry,
                args.first().cloned().unwrap_or(JsValue::Undefined),
                trusted,
            )
        }),
    );
}
