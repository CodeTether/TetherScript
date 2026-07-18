use super::*;

pub(super) fn replace(text: &str) -> Result<bool, String> {
    let Some(handle) = selection_host::command::focused_editable() else {
        return Ok(false);
    };
    handle.insert_text_at_selection(text)?;
    Ok(true)
}

pub(super) fn select_all(document: &DomHandle) -> Result<bool, String> {
    let Some(handle) = selection_host::command::focused_editable() else {
        return Ok(false);
    };
    if !selection_host::command::select_all(&handle) {
        return Ok(false);
    }
    handle.dispatch_event(JsValue::String("select".into()))?;
    document.dispatch_event(JsValue::String("selectionchange".into()))?;
    Ok(true)
}
