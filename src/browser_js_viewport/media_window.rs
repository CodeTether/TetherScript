use super::*;

#[path = "media_window_sync.rs"]
mod sync;
#[cfg(test)]
#[path = "tests_media_resize.rs"]
mod tests_resize;

pub(super) struct State {
    width: i64,
    queries: Vec<JsValue>,
}

pub(super) fn install(window: &mut HashMap<String, JsValue>) {
    let state = Rc::new(RefCell::new(State {
        width: constants::DEFAULT_VIEWPORT_WIDTH,
        queries: Vec::new(),
    }));
    let query_state = state.clone();
    window.insert(
        "matchMedia".into(),
        native("matchMedia", Some(1), move |args| {
            let query = args.first().map(JsValue::display).unwrap_or_default();
            let object = media_object::create(query, query_state.borrow().width);
            query_state.borrow_mut().queries.push(object.clone());
            Ok(object)
        }),
    );
    window.insert(
        "__tsSyncMedia".into(),
        native("__tsSyncMedia", Some(1), move |args| {
            let width = args
                .first()
                .map(JsValue::display)
                .and_then(|value| value.parse().ok())
                .unwrap_or(0);
            sync::apply(&state, width)?;
            Ok(JsValue::Undefined)
        }),
    );
}
