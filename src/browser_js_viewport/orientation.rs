use super::super::*;

#[path = "orientation_event.rs"]
mod event;
#[path = "orientation_events.rs"]
mod events;
#[path = "orientation_lock.rs"]
mod lock;
#[path = "orientation_state.rs"]
mod state;
#[cfg(test)]
#[path = "tests_orientation_events.rs"]
mod tests_orientation_events;
#[cfg(test)]
#[path = "tests_orientation_lock.rs"]
mod tests_orientation_lock;
#[path = "orientation_update.rs"]
mod update;
#[path = "orientation_value.rs"]
mod value;
#[path = "orientation_viewport_state.rs"]
mod viewport_state;

pub(super) fn object() -> JsValue {
    let object = Rc::new(RefCell::new(HashMap::from([
        ("type".into(), JsValue::String("landscape-primary".into())),
        ("angle".into(), JsValue::Number(0.0)),
        ("onchange".into(), JsValue::Null),
    ])));
    viewport_state::initialize(&object);
    events::install(&object);
    lock::install(&object);
    JsValue::Object(object)
}

pub(super) fn sync(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    width: f64,
    height: f64,
) -> Result<(), String> {
    viewport_state::remember(object, width, height);
    if state::follows_viewport(object) {
        update::apply(object, value::viewport(width, height))?;
    }
    Ok(())
}
