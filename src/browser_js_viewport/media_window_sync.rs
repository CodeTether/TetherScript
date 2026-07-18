//! Live MediaQueryList refresh after viewport changes.

use super::super::*;

pub(super) fn apply(state: &Rc<RefCell<super::State>>, width: i64) -> Result<(), String> {
    let queries = {
        let mut state = state.borrow_mut();
        state.width = width;
        state.queries.clone()
    };
    for query in queries {
        refresh(query, width)?;
    }
    Ok(())
}

fn refresh(query: JsValue, width: i64) -> Result<(), String> {
    let JsValue::Object(object) = &query else {
        return Ok(());
    };
    let media = object
        .borrow()
        .get("media")
        .map(JsValue::display)
        .unwrap_or_default();
    let next = super::super::media_query::matches(&media, width);
    let previous = object.borrow().get("matches").is_some_and(JsValue::truthy);
    if next == previous {
        return Ok(());
    }
    object
        .borrow_mut()
        .insert("matches".into(), JsValue::Bool(next));
    let dispatcher = object.borrow().get("dispatchEvent").cloned();
    let event = event(next, media);
    if let Some(dispatcher) = dispatcher {
        js::call_function_with_this(dispatcher, query, &[event])?;
    }
    Ok(())
}

fn event(matches: bool, media: String) -> JsValue {
    JsValue::Object(Rc::new(RefCell::new(HashMap::from([
        ("type".into(), JsValue::String("change".into())),
        ("matches".into(), JsValue::Bool(matches)),
        ("media".into(), JsValue::String(media)),
        ("isTrusted".into(), JsValue::Bool(true)),
    ]))))
}
