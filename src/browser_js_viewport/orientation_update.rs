use super::super::*;

pub(super) fn apply(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    state: super::value::Snapshot,
) -> Result<bool, String> {
    let changed = {
        let object = object.borrow();
        object.get("type").map(JsValue::display).as_deref() != Some(state.kind)
            || object.get("angle") != Some(&JsValue::Number(state.angle))
    };
    if !changed {
        return Ok(false);
    }
    {
        let mut object = object.borrow_mut();
        object.insert("type".into(), JsValue::String(state.kind.into()));
        object.insert("angle".into(), JsValue::Number(state.angle));
    }
    super::events::dispatch_change(object)?;
    Ok(true)
}
