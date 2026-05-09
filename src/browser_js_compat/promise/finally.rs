use super::*;

pub(super) fn method(state: Rc<RefCell<state::PromiseState>>) -> JsValue {
    native("Promise.finally", None, move |args| {
        let callback = args.first().cloned().unwrap_or(JsValue::Undefined);
        let current = state.borrow().clone();
        let next = if handler::present(&callback) {
            after_callback(callback, current)
        } else {
            current
        };
        Ok(object::from_state(next))
    })
}

fn after_callback(callback: JsValue, current: state::PromiseState) -> state::PromiseState {
    match handler::invoke(callback, &[]) {
        state::PromiseState::Rejected(reason) => state::PromiseState::Rejected(reason),
        _ => current,
    }
}
