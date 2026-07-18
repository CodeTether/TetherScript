use super::*;

pub(super) fn install(object: &Rc<RefCell<HashMap<String, JsValue>>>, registry: model::Registry) {
    listener_add::install(object, registry.clone());
    listener_remove::install(object, registry);
}

pub(super) fn remove(
    registry: &model::Registry,
    event_type: &str,
    callback: &JsValue,
    capture: bool,
) {
    listener_remove::remove(registry, event_type, callback, capture);
}

pub(super) fn contains(
    registry: &model::Registry,
    event_type: &str,
    callback: &JsValue,
    capture: bool,
) -> bool {
    listener_remove::contains(registry, event_type, callback, capture)
}
