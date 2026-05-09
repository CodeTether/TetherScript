use super::*;

pub(super) fn install(document: &JsValue, root: Rc<RefCell<Node>>) {
    let JsValue::Object(object) = document else {
        return;
    };
    let single_root = root.clone();
    let all_root = root;
    let mut object = object.borrow_mut();
    object.insert("hidden".into(), JsValue::Bool(false));
    object.insert("visibilityState".into(), JsValue::String("visible".into()));
    object.insert(
        "elementFromPoint".into(),
        native("document.elementFromPoint", Some(2), move |args| {
            let (x, y) = point(args);
            Ok(hit::single(&single_root, x, y))
        }),
    );
    object.insert(
        "elementsFromPoint".into(),
        native("document.elementsFromPoint", Some(2), move |args| {
            let (x, y) = point(args);
            Ok(hit::all(&all_root, x, y))
        }),
    );
}

fn point(args: &[JsValue]) -> (i64, i64) {
    (number(args.first()), number(args.get(1)))
}

fn number(value: Option<&JsValue>) -> i64 {
    value
        .map(JsValue::display)
        .and_then(|raw| raw.parse::<f64>().ok())
        .filter(|value| value.is_finite())
        .unwrap_or(0.0)
        .trunc() as i64
}
