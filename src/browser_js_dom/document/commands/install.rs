use super::*;

pub(super) fn methods(object: &mut HashMap<String, JsValue>, document: &DomHandle) {
    let document = document.clone();
    object.insert(
        "execCommand".into(),
        native("document.execCommand", None, move |args| {
            let Some(command) = command::Command::parse(args.first()) else {
                return Ok(JsValue::Bool(false));
            };
            let value = args.get(2).map(JsValue::display).unwrap_or_default();
            Ok(JsValue::Bool(execute::run(&document, command, &value)?))
        }),
    );
    object.insert(
        "queryCommandSupported".into(),
        native("document.queryCommandSupported", Some(1), |args| {
            Ok(JsValue::Bool(
                command::Command::parse(args.first()).is_some(),
            ))
        }),
    );
    object.insert(
        "queryCommandEnabled".into(),
        native("document.queryCommandEnabled", Some(1), |args| {
            let enabled = command::Command::parse(args.first()).is_some_and(query::enabled);
            Ok(JsValue::Bool(enabled))
        }),
    );
}
