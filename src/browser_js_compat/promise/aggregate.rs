use super::*;

pub(super) fn all(args: &[JsValue]) -> Result<JsValue, String> {
    let items = array_arg(args, "Promise.all")?;
    let mut values = Vec::new();
    for item in items.borrow().iter().cloned() {
        match state::settle(item) {
            state::PromiseState::Fulfilled(value) => values.push(value),
            state::PromiseState::Rejected(reason) => return Ok(rejected(reason)),
            state::PromiseState::Pending => return Ok(pending()),
        }
    }
    Ok(fulfilled(JsValue::Array(Rc::new(RefCell::new(values)))))
}

pub(super) fn race(args: &[JsValue]) -> Result<JsValue, String> {
    let items = array_arg(args, "Promise.race")?;
    let first = items.borrow().first().cloned();
    Ok(first
        .map(|value| object::from_state(state::settle(value)))
        .unwrap_or_else(pending))
}

fn array_arg(args: &[JsValue], name: &str) -> Result<Rc<RefCell<Vec<JsValue>>>, String> {
    match args.first().cloned().unwrap_or(JsValue::Undefined) {
        JsValue::Array(items) => Ok(items),
        other => Err(format!("{name}: expected array, got {}", other.display())),
    }
}
