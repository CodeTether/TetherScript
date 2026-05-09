use super::*;

pub(super) fn install(window: &mut HashMap<String, JsValue>) {
    let object = Rc::new(RefCell::new(HashMap::new()));
    {
        let mut map = object.borrow_mut();
        install_metrics(&mut map);
        install_events(&mut map);
    }
    window.insert("visualViewport".into(), JsValue::Object(object));
}

fn install_metrics(map: &mut HashMap<String, JsValue>) {
    for (name, value) in [
        ("width", constants::DEFAULT_VIEWPORT_WIDTH as f64),
        ("height", constants::DEFAULT_VIEWPORT_HEIGHT as f64),
        ("scale", constants::DEVICE_PIXEL_RATIO),
        ("offsetLeft", 0.0),
        ("offsetTop", 0.0),
        ("pageLeft", 0.0),
        ("pageTop", 0.0),
    ] {
        map.insert(name.into(), JsValue::Number(value));
    }
}

fn install_events(map: &mut HashMap<String, JsValue>) {
    map.insert(
        "addEventListener".into(),
        native("visualViewport.addEventListener", None, |_| {
            Ok(JsValue::Undefined)
        }),
    );
    map.insert(
        "removeEventListener".into(),
        native("visualViewport.removeEventListener", None, |_| {
            Ok(JsValue::Undefined)
        }),
    );
    map.insert(
        "dispatchEvent".into(),
        native("visualViewport.dispatchEvent", Some(1), |_| {
            Ok(JsValue::Bool(true))
        }),
    );
}
