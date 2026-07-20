use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: &DomHandle) {
    for property in ["data", "nodeValue"] {
        let getter = handle.clone();
        obj.insert(
            format!("__get:{property}"),
            native(property, Some(0), move |_| {
                Ok(JsValue::String(state::read(&getter)))
            }),
        );
        let setter = handle.clone();
        obj.insert(
            format!("__set:{property}"),
            native(property, Some(1), move |args| {
                let old = state::read(&setter);
                state::write(&setter, old, args::text(args, 0));
                Ok(JsValue::Undefined)
            }),
        );
    }
    let getter = handle.clone();
    obj.insert(
        "__get:length".into(),
        native("CharacterData.length", Some(0), move |_| {
            Ok(JsValue::Number(units::length(&state::read(&getter)) as f64))
        }),
    );
}
