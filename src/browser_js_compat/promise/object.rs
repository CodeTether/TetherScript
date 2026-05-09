use super::*;

pub(super) fn from_state(state: state::PromiseState) -> JsValue {
    let shared = Rc::new(RefCell::new(state));
    let mut object = HashMap::new();
    write_state(&mut object, &shared.borrow());
    install_methods(&mut object, shared);
    JsValue::Object(Rc::new(RefCell::new(object)))
}

pub(super) fn install_methods(
    object: &mut HashMap<String, JsValue>,
    state: Rc<RefCell<state::PromiseState>>,
) {
    object.insert("then".into(), then::method(state.clone()));
    object.insert("catch".into(), catch::method(state.clone()));
    object.insert("finally".into(), finally::method(state));
}

pub(super) fn write_state(object: &mut HashMap<String, JsValue>, state: &state::PromiseState) {
    object.remove("__promise_value");
    object.remove("__promise_reason");
    object.remove("value");
    object.remove("reason");
    match state {
        state::PromiseState::Pending => {
            object.insert("__promise_state".into(), JsValue::String("pending".into()));
        }
        state::PromiseState::Fulfilled(value) => {
            object.insert(
                "__promise_state".into(),
                JsValue::String("fulfilled".into()),
            );
            object.insert("__promise_value".into(), value.clone());
            object.insert("value".into(), value.clone());
        }
        state::PromiseState::Rejected(reason) => {
            object.insert("__promise_state".into(), JsValue::String("rejected".into()));
            object.insert("__promise_reason".into(), reason.clone());
            object.insert("reason".into(), reason.clone());
        }
    }
}
