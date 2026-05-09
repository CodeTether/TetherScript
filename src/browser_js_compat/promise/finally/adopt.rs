use super::{super::*, action::Action};

#[path = "callbacks.rs"]
mod callbacks;
#[path = "link.rs"]
mod link;

pub(super) fn from_action(action: Action) -> Result<JsValue, String> {
    match action {
        Action::Ready(state) => Ok(object::from_state(state)),
        Action::Wait(value, current) => new_promise(&value, current),
    }
}

fn new_promise(returned: &JsValue, original: state::PromiseState) -> Result<JsValue, String> {
    let state = Rc::new(RefCell::new(state::PromiseState::Pending));
    let queue = reaction::queue();
    let object = object::from_parts(state.clone(), queue.clone());
    link::attach(returned, original, state, object.clone(), queue)?;
    Ok(JsValue::Object(object))
}

pub(super) fn settle_into(
    action: Action,
    state: Rc<RefCell<state::PromiseState>>,
    object: Rc<RefCell<HashMap<String, JsValue>>>,
    queue: reaction::Queue,
) {
    match action {
        Action::Ready(next) => reaction::settle(&state, &object, &queue, next),
        Action::Wait(value, original) => {
            if let Err(error) = link::attach(
                &value,
                original,
                state.clone(),
                object.clone(),
                queue.clone(),
            ) {
                reaction::settle(
                    &state,
                    &object,
                    &queue,
                    state::PromiseState::Rejected(JsValue::String(error)),
                );
            }
        }
    }
}
