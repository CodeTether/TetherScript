//! Keyboard actions routed to the host's focused locator.

use crate::value::Value;

use super::state::HostState;

#[path = "keyboard_type.rs"]
mod keyboard_type;
#[cfg(test)]
#[path = "keyboard_type_tests.rs"]
mod keyboard_type_tests;

pub(super) fn invoke(
    state: &mut HostState,
    action: &str,
    payload: &Value,
) -> Result<Value, String> {
    let locator = state.focused.clone().ok_or_else(|| {
        format!("browser.{action}: no focused element; call browser.focus, click, or fill first")
    })?;
    let report = if action == "keyboard_type" {
        keyboard_type::invoke(state, &locator, payload)?
    } else {
        let key = super::value::string_field(payload, "key")?;
        state.page.press(&locator, key.as_str())?
    };
    Ok(super::interact_value::report(report))
}
