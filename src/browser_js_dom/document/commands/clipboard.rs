use super::*;

use compat_host::events::data_transfer::Transfer;

pub(super) fn copy(document: &DomHandle) -> Result<bool, String> {
    let selected = selection_host::command::text();
    let transfer = Transfer::new("");
    let allowed = dispatch(document, "copy", &transfer)?;
    state::write(if allowed { selected } else { transfer.text() });
    Ok(true)
}

pub(super) fn cut(document: &DomHandle) -> Result<bool, String> {
    let selected = selection_host::command::text();
    let transfer = Transfer::new("");
    let allowed = dispatch(document, "cut", &transfer)?;
    state::write(if allowed { selected } else { transfer.text() });
    if allowed {
        edit::replace("")?;
    }
    Ok(true)
}

pub(super) fn paste(document: &DomHandle) -> Result<bool, String> {
    let transfer = Transfer::new(&state::read().unwrap_or_default());
    if dispatch(document, "paste", &transfer)? {
        edit::replace(&transfer.text())?;
    }
    Ok(true)
}

fn dispatch(document: &DomHandle, kind: &str, transfer: &Transfer) -> Result<bool, String> {
    let target = selection_host::command::focused().unwrap_or_else(|| document.clone());
    let mut fields = HashMap::new();
    fields.insert("type".into(), JsValue::String(kind.into()));
    fields.insert("bubbles".into(), JsValue::Bool(true));
    fields.insert("cancelable".into(), JsValue::Bool(true));
    fields.insert("clipboardData".into(), transfer.value());
    Ok(target
        .dispatch_event(JsValue::Object(Rc::new(RefCell::new(fields))))?
        .truthy())
}
