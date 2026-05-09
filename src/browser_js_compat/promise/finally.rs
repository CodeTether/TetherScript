use super::*;

pub(super) fn method(
    state: Rc<RefCell<state::PromiseState>>,
    reactions: reaction::Queue,
) -> JsValue {
    native("Promise.finally", None, move |args| {
        let callback = args.first().cloned().unwrap_or(JsValue::Undefined);
        let current = state.borrow().clone();
        if matches!(current, state::PromiseState::Pending) {
            return Ok(reaction::push_finally(reactions.clone(), callback));
        }
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

pub(super) fn settle(callback: JsValue, current: state::PromiseState) -> state::PromiseState {
    if handler::present(&callback) {
        after_callback(callback, current)
    } else {
        current
    }
}
