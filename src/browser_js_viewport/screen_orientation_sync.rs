use super::super::*;

pub(super) fn sync(window: &JsValue) -> Result<bool, String> {
    let JsValue::Object(window) = window else {
        return Ok(false);
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
        return Ok(false);
    };
    let orientation = screen.borrow().get("orientation").cloned();
    let Some(JsValue::Object(orientation)) = orientation else {
        return Ok(false);
    };
    let changed = super::orientation::sync(&orientation, width, height)?;
    let angle = orientation.borrow().get("angle").cloned();
    if let Some(angle) = angle {
        window.borrow_mut().insert("orientation".into(), angle);
    }
    Ok(changed)
}

fn number(value: Option<&JsValue>) -> f64 {
    value
        .map(JsValue::display)
        .and_then(|value| value.parse().ok())
        .filter(|value: &f64| value.is_finite())
        .unwrap_or(0.0)
}
