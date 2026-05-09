use super::*;

pub(super) fn method(
    state: Rc<RefCell<state::PromiseState>>,
    reactions: reaction::Queue,
) -> JsValue {
    native("Promise.then", None, move |args| {
        let on_ok = args.first().cloned().unwrap_or(JsValue::Undefined);
        let on_err = args.get(1).cloned().unwrap_or(JsValue::Undefined);
        let next = match state.borrow().clone() {
            state::PromiseState::Fulfilled(value) => fulfilled(on_ok, value),
            state::PromiseState::Rejected(reason) => rejected(on_err, reason),
            state::PromiseState::Pending => {
                return Ok(reaction::push_then(reactions.clone(), on_ok, on_err))
            }
        };
        Ok(object::from_state(next))
    })
}

fn fulfilled(handler_value: JsValue, value: JsValue) -> state::PromiseState {
    if handler::present(&handler_value) {
        handler::invoke(handler_value, &[value])
    } else {
        state::PromiseState::Fulfilled(value)
    }
}

fn rejected(handler_value: JsValue, reason: JsValue) -> state::PromiseState {
    if handler::present(&handler_value) {
        handler::invoke(handler_value, &[reason])
    } else {
        state::PromiseState::Rejected(reason)
    }
}

pub(super) fn settle(
    ok: JsValue,
    err: JsValue,
    current: state::PromiseState,
) -> state::PromiseState {
    match current {
        state::PromiseState::Fulfilled(value) => fulfilled(ok, value),
        state::PromiseState::Rejected(reason) => rejected(err, reason),
        state::PromiseState::Pending => state::PromiseState::Pending,
    }
}
