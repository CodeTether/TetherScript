use super::*;

pub(super) fn install(object: &Rc<RefCell<HashMap<String, JsValue>>>) {
    insert_lock(object);
    insert_unlock(object);
}

fn insert_lock(object: &Rc<RefCell<HashMap<String, JsValue>>>) {
    let target = object.clone();
    object.borrow_mut().insert(
        "lock".into(),
        native("screen.orientation.lock", Some(1), move |args| {
            let kind = args.first().map(JsValue::display).unwrap_or_default();
            let current = viewport_state::current(&target);
            let requested = match value::requested(&kind, current) {
                Ok(requested) => requested,
                Err(reason) => return Ok(rejected(reason)),
            };
            state::set_lock(&target, Some(&kind));
            update::apply(&target, requested)?;
            Ok(fulfilled())
        }),
    );
}

fn insert_unlock(object: &Rc<RefCell<HashMap<String, JsValue>>>) {
    let target = object.clone();
    object.borrow_mut().insert(
        "unlock".into(),
        native("screen.orientation.unlock", Some(0), move |_| {
            state::set_lock(&target, None);
            update::apply(&target, viewport_state::current(&target))?;
            Ok(JsValue::Undefined)
        }),
    );
}

fn fulfilled() -> JsValue {
    compat_host::promise::api::fulfilled(JsValue::Undefined)
}

fn rejected(reason: String) -> JsValue {
    compat_host::promise::api::rejected(JsValue::String(reason))
}
