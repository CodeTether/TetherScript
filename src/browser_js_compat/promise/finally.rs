use super::*;

#[path = "finally/action.rs"]
mod action;
#[path = "finally/adopt.rs"]
mod adopt;

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
        if !handler::present(&callback) {
            return Ok(object::from_state(current));
        }
        adopt::from_action(action::after_callback(callback, current))
    })
}

pub(super) fn settle_reaction(
    callback: JsValue,
    current: state::PromiseState,
    state: Rc<RefCell<state::PromiseState>>,
    object: Rc<RefCell<HashMap<String, JsValue>>>,
    queue: reaction::Queue,
) {
    if !handler::present(&callback) {
        reaction::settle(&state, &object, &queue, current);
        return;
    }
    adopt::settle_into(
        action::after_callback(callback, current),
        state,
        object,
        queue,
    );
}
