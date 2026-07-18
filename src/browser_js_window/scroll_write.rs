use super::*;

pub(super) fn to(x: i64, y: i64) -> Result<bool, String> {
    let Some(window) = state::current() else {
        return Ok(false);
    };
    let next = (x.max(0), y.max(0));
    let old = metrics();
    if (old.x, old.y) == next {
        return Ok(false);
    }
    let scroll_to = window.borrow().get("scrollTo").cloned();
    if let Some(scroll_to) = scroll_to {
        js::call_function_with_this(
            scroll_to,
            JsValue::Object(window),
            &[
                JsValue::Number(next.0 as f64),
                JsValue::Number(next.1 as f64),
            ],
        )?;
        return Ok(true);
    }
    for (name, value) in [
        ("scrollX", next.0),
        ("pageXOffset", next.0),
        ("scrollY", next.1),
        ("pageYOffset", next.1),
    ] {
        window
            .borrow_mut()
            .insert(name.into(), JsValue::Number(value as f64));
    }
    dispatch_window_lifecycle(&JsValue::Object(window), "scroll")?;
    Ok(true)
}
