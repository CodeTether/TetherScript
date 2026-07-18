use super::*;

#[path = "media_window_css.rs"]
mod css;
#[path = "media_window_sync.rs"]
mod sync;
#[cfg(test)]
#[path = "tests_media_resize.rs"]
mod tests_resize;

pub(super) struct State {
    css_source: String,
    width: i64,
    queries: Vec<JsValue>,
}

pub(super) fn install(window: &mut HashMap<String, JsValue>) {
    let css_source = super::super::LAYOUT_CSS.with(|source| source.borrow().clone());
    css::apply(&css_source, constants::DEFAULT_VIEWPORT_WIDTH);
    let state = Rc::new(RefCell::new(State {
        css_source,
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
                .and_then(|value| value.display().parse().ok())
                .unwrap_or(0);
            css::apply(&state.borrow().css_source, width);
            sync::apply(&state, width)?;
            Ok(JsValue::Undefined)
        }),
    );
}
