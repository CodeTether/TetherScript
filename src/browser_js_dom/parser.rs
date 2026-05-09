use super::*;

pub(super) fn install(window: &mut HashMap<String, JsValue>) {
    window.insert(
        "DOMParser".into(),
        native("DOMParser", Some(0), move |_| Ok(parser_object())),
    );
}

fn parser_object() -> JsValue {
    let mut obj = HashMap::new();
    obj.insert(
        "parseFromString".into(),
        native("DOMParser.parseFromString", Some(2), move |args| {
            let html = args.first().unwrap_or(&JsValue::Undefined).display();
            let mime = args.get(1).unwrap_or(&JsValue::Undefined).display();
            let source = if mime == "text/html" {
                html
            } else {
                String::new()
            };
            Ok(construct::document_from_html(&source))
        }),
    );
    JsValue::Object(Rc::new(RefCell::new(obj)))
}
