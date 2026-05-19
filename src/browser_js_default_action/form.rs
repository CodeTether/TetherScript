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
        "reset" => return reset_closest(handle),
        _ => {}
    }
    Ok(true)
}

pub(super) fn button(handle: &DomHandle, el: &Element) -> Result<bool, String> {
    match control_type(el, "submit").as_str() {
        "submit" => submit_closest(handle),
        "reset" => reset_closest(handle),
        _ => Ok(true),
    }
}

fn control_type(el: &Element, default: &str) -> String {
    el.attrs
        .get("type")
        .map(|t| t.to_ascii_lowercase())
        .unwrap_or_else(|| default.into())
}

fn submit_closest(handle: &DomHandle) -> Result<bool, String> {
    let Some(form) = handle.closest_form() else {
        return Ok(true);
    };
    Ok(submit_form(&form, true, Some(handle))?.truthy())
}

fn reset_closest(handle: &DomHandle) -> Result<bool, String> {
    let Some(form) = handle.closest_form() else {
        return Ok(true);
    };
    if form
        .dispatch_event(JsValue::String("reset".into()))?
        .truthy()
    {
        super::form_reset::perform(&form)?;
    }
    Ok(true)
}

fn dispatch_input_change(handle: &DomHandle) -> Result<(), String> {
    handle.dispatch_event(JsValue::String("input".into()))?;
    handle.dispatch_event(JsValue::String("change".into()))?;
    Ok(())
}
