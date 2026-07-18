use super::*;

pub(super) fn install(map: &mut HashMap<String, JsValue>) {
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

pub(super) fn sync(
    window: &Rc<RefCell<HashMap<String, JsValue>>>,
    viewport: &Rc<RefCell<HashMap<String, JsValue>>>,
) {
    let window = window.borrow();
    let values = [
        ("width", number(window.get("innerWidth"))),
        ("height", number(window.get("innerHeight"))),
        ("pageLeft", number(window.get("scrollX"))),
        ("pageTop", number(window.get("scrollY"))),
    ];
    drop(window);
    for (name, value) in values {
        viewport
            .borrow_mut()
            .insert(name.into(), JsValue::Number(value));
    }
}

fn number(value: Option<&JsValue>) -> f64 {
    value
        .and_then(|value| value.display().parse().ok())
        .filter(|value: &f64| value.is_finite())
        .unwrap_or(0.0)
}
