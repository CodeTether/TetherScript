use super::*;

pub(super) fn method(
    state: Rc<RefCell<state::PromiseState>>,
    reactions: reaction::Queue,
) -> JsValue {
    native("Promise.catch", None, move |args| {
        let handler_value = args.first().cloned().unwrap_or(JsValue::Undefined);
        let next = match state.borrow().clone() {
            state::PromiseState::Rejected(reason) if handler::present(&handler_value) => {
                handler::invoke(handler_value, &[reason])
            }
            state::PromiseState::Rejected(reason) => state::PromiseState::Rejected(reason),
            state::PromiseState::Fulfilled(value) => state::PromiseState::Fulfilled(value),
            state::PromiseState::Pending => {
                return Ok(reaction::push_then(
                    reactions.clone(),
                    JsValue::Undefined,
                    handler_value,
                ));
            }
        };
        Ok(object::from_state(next))
    })
}
