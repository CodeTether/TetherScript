use super::*;

pub(super) fn construct(args: &[JsValue]) -> Result<JsValue, String> {
    let state = Rc::new(RefCell::new(state::PromiseState::Pending));
    let object = Rc::new(RefCell::new(HashMap::new()));
    {
        let mut map = object.borrow_mut();
        object::write_state(&mut map, &state.borrow());
        object::install_methods(&mut map, state.clone());
    }
    let resolve = settler(state.clone(), object.clone(), true);
    let reject = settler(state.clone(), object.clone(), false);
    let executor = args.first().cloned().unwrap_or(JsValue::Undefined);
    if let Err(error) =
        js::call_function_with_this(executor, JsValue::Undefined, &[resolve, reject])
    {
        settle(
            &state,
            &object,
            state::PromiseState::Rejected(JsValue::String(error)),
        );
    }
    Ok(JsValue::Object(object))
}

fn settler(
    state: Rc<RefCell<state::PromiseState>>,
    object: Rc<RefCell<HashMap<String, JsValue>>>,
    fulfill: bool,
) -> JsValue {
    native("Promise.settle", Some(1), move |args| {
        let value = args.first().cloned().unwrap_or(JsValue::Undefined);
        let next = if fulfill {
            state::settle(value)
        } else {
            state::PromiseState::Rejected(value)
        };
        settle(&state, &object, next);
        Ok(JsValue::Undefined)
    })
}

fn settle(
    state: &Rc<RefCell<state::PromiseState>>,
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    next: state::PromiseState,
) {
    if !matches!(*state.borrow(), state::PromiseState::Pending) {
        return;
    }
    *state.borrow_mut() = next;
    object::write_state(&mut object.borrow_mut(), &state.borrow());
}
