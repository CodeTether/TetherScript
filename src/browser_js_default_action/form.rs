//! Form-control click default actions.

use super::super::*;

pub(super) fn input(handle: &DomHandle, el: &Element) -> Result<bool, String> {
    match control_type(el, "text").as_str() {
        "checkbox" => {
            handle.set_checked_state(!el.attrs.contains_key("checked"));
            dispatch_input_change(handle)?;
        }
        "radio" => {
            handle.set_checked_state(true);
            dispatch_input_change(handle)?;
        }
        "submit" => return submit_closest(handle),
        _ => {}
    }
    Ok(true)
}

pub(super) fn button(handle: &DomHandle, el: &Element) -> Result<bool, String> {
    if control_type(el, "submit") == "submit" {
        return submit_closest(handle);
    }
    Ok(true)
}

fn control_type(el: &Element, default: &str) -> String {
    el.attrs
        .get("type")
        .map(|ty| ty.to_ascii_lowercase())
        .unwrap_or_else(|| default.into())
}

fn submit_closest(handle: &DomHandle) -> Result<bool, String> {
    let Some(form) = handle.closest_form() else {
        return Ok(true);
    };
    Ok(submit_form(&form, true)?.truthy())
}

fn dispatch_input_change(handle: &DomHandle) -> Result<(), String> {
    handle.dispatch_event(JsValue::String("input".into()))?;
    handle.dispatch_event(JsValue::String("change".into()))?;
    Ok(())
}
