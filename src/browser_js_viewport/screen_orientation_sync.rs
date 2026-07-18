use super::super::*;

pub(super) fn sync(window: &JsValue) -> Result<(), String> {
    let JsValue::Object(window) = window else {
        return Ok(());
    };
    let (width, height, screen) = {
        let window = window.borrow();
        (
            number(window.get("innerWidth")),
            number(window.get("innerHeight")),
            window.get("screen").cloned(),
        )
    };
    let Some(JsValue::Object(screen)) = screen else {
        return Ok(());
    };
    let orientation = screen.borrow().get("orientation").cloned();
    let Some(JsValue::Object(orientation)) = orientation else {
        return Ok(());
    };
    super::orientation::sync(&orientation, width, height)
}

fn number(value: Option<&JsValue>) -> f64 {
    value
        .map(JsValue::display)
        .and_then(|value| value.parse().ok())
        .filter(|value: &f64| value.is_finite())
        .unwrap_or(0.0)
}
