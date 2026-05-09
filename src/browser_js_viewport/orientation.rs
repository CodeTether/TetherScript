use super::super::*;

pub(super) fn object() -> JsValue {
    let mut object = HashMap::from([
        ("type".into(), JsValue::String("landscape-primary".into())),
        ("angle".into(), JsValue::Number(0.0)),
        ("onchange".into(), JsValue::Null),
    ]);
    object.insert("addEventListener".into(), noop("addEventListener"));
    object.insert("removeEventListener".into(), noop("removeEventListener"));
    object.insert(
        "dispatchEvent".into(),
        native("screen.orientation.dispatchEvent", Some(1), |_| {
            Ok(JsValue::Bool(true))
        }),
    );
    JsValue::Object(Rc::new(RefCell::new(object)))
}

fn noop(name: &str) -> JsValue {
    native(&format!("screen.orientation.{name}"), None, |_| {
        Ok(JsValue::Undefined)
    })
}
