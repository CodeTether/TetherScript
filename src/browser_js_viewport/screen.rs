use super::*;

#[path = "orientation.rs"]
mod orientation;
#[path = "screen_orientation_sync.rs"]
mod orientation_sync;

pub(super) fn install(window: &mut HashMap<String, JsValue>) {
    window.insert(
        "innerWidth".into(),
        JsValue::Number(constants::DEFAULT_VIEWPORT_WIDTH as f64),
    );
    window.insert(
        "innerHeight".into(),
        JsValue::Number(constants::DEFAULT_VIEWPORT_HEIGHT as f64),
    );
    window.insert(
        "devicePixelRatio".into(),
        JsValue::Number(constants::DEVICE_PIXEL_RATIO),
    );
    window.insert("orientation".into(), JsValue::Number(0.0));
    window.insert("screen".into(), screen_object());
}

pub(in crate::browser_js) fn sync_orientation(window: &JsValue) -> Result<(), String> {
    orientation_sync::sync(window)
}

fn screen_object() -> JsValue {
    let mut object = HashMap::new();
    for (name, value) in [
        ("width", constants::DEFAULT_VIEWPORT_WIDTH),
        ("height", constants::DEFAULT_VIEWPORT_HEIGHT),
        ("availWidth", constants::DEFAULT_VIEWPORT_WIDTH),
        ("availHeight", constants::DEFAULT_VIEWPORT_HEIGHT),
        ("colorDepth", 24),
        ("pixelDepth", 24),
    ] {
        object.insert(name.into(), JsValue::Number(value as f64));
    }
    object.insert("orientation".into(), orientation::object());
    JsValue::Object(Rc::new(RefCell::new(object)))
}
