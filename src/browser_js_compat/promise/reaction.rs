use super::*;

#[path = "reaction/apply.rs"]
mod apply;
#[path = "reaction/types.rs"]
mod types;

pub(super) type Queue = Rc<RefCell<Vec<types::Reaction>>>;

pub(super) fn queue() -> Queue {
    Rc::new(RefCell::new(Vec::new()))
}

pub(super) fn push_then(queue: Queue, ok: JsValue, err: JsValue) -> JsValue {
    push(queue, types::Kind::Then { ok, err })
}

pub(super) fn push_finally(queue: Queue, callback: JsValue) -> JsValue {
    push(queue, types::Kind::Finally { callback })
}

fn push(queue: Queue, kind: types::Kind) -> JsValue {
    let state = Rc::new(RefCell::new(state::PromiseState::Pending));
    let child_queue = self::queue();
    let object = object::from_parts(state.clone(), child_queue.clone());
    queue.borrow_mut().push(types::Reaction {
        kind,
        state,
        object: object.clone(),
        queue: child_queue,
    });
    JsValue::Object(object)
}

pub(super) fn settle(
    state: &Rc<RefCell<state::PromiseState>>,
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    queue: &Queue,
    next: state::PromiseState,
) {
    if !matches!(*state.borrow(), state::PromiseState::Pending) {
        return;
    }
    *state.borrow_mut() = next;
    object::write_state(&mut object.borrow_mut(), &state.borrow());
    deliver(queue, state.borrow().clone());
}

fn deliver(queue: &Queue, current: state::PromiseState) {
    let reactions = queue.borrow_mut().drain(..).collect::<Vec<_>>();
    for item in reactions {
        apply::settle(item, current.clone());
    }
}
