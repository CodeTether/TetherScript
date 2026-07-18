use super::*;

pub(super) fn current(handle: &DomHandle) -> state::Position {
    let geometry = geometry::measure(handle);
    let raw = state::get(handle);
    let clamped = state::Position {
        left: raw.left.clamp(0, geometry.max_left()),
        top: raw.top.clamp(0, geometry.max_top()),
    };
    if raw != clamped {
        state::set(handle, clamped);
    }
    clamped
}

pub(super) fn to(handle: &DomHandle, requested: state::Position) -> Result<bool, String> {
    let geometry = geometry::measure(handle);
    let next = state::Position {
        left: requested.left.clamp(0, geometry.max_left()),
        top: requested.top.clamp(0, geometry.max_top()),
    };
    if current(handle) == next {
        return Ok(false);
    }
    state::set(handle, next);
    handle.dispatch_event(scroll_event())?;
    Ok(true)
}

pub(super) fn axis(handle: &DomHandle, name: &str, value: i64) -> Result<(), String> {
    let mut next = current(handle);
    if name == "scrollLeft" {
        next.left = value;
    } else {
        next.top = value;
    }
    to(handle, next).map(|_| ())
}

fn scroll_event() -> JsValue {
    let mut event = HashMap::new();
    event.insert("type".into(), JsValue::String("scroll".into()));
    event.insert("bubbles".into(), JsValue::Bool(false));
    event.insert("cancelable".into(), JsValue::Bool(false));
    JsValue::Object(Rc::new(RefCell::new(event)))
}
