//! Keyboard actions routed to the host's focused locator.

use crate::value::Value;

use super::state::HostState;

pub(super) fn invoke(state: &mut HostState, payload: &Value) -> Result<Value, String> {
    let key = super::value::string_field(payload, "key")?;
    let locator = state.focused.clone().ok_or_else(|| {
        "browser.press: no focused element; call browser.focus, click, or fill first".to_string()
    })?;
    let report = state.page.press(&locator, key.as_str())?;
    Ok(super::interact_value::report(report))
}
