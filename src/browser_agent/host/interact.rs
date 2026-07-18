//! Native locator-backed click, fill, type, and hover actions.

use crate::browser_agent::Locator;
use crate::value::Value;

use super::state::HostState;

#[path = "fill.rs"]
mod fill;
#[path = "toggle.rs"]
mod toggle;
#[cfg(test)]
#[path = "type_tests.rs"]
mod type_tests;
#[path = "type_text.rs"]
mod type_text;
#[path = "upload.rs"]
mod upload;
#[cfg(test)]
#[path = "upload_tests.rs"]
mod upload_tests;

pub(super) fn invoke(
    state: &mut HostState,
    action: &str,
    payload: &Value,
) -> Result<Value, String> {
    let report = match action {
        "click" => state.page.click(&Locator::css(super::value::string_field(
            payload, "selector",
        )?))?,
        "click_text" => state
            .page
            .click(&Locator::text(super::value::string_field(payload, "text")?))?,
        "fill" => fill::invoke(state, payload)?,
        "type" => type_text::invoke(state, payload)?,
        "upload" => upload::invoke(state, payload)?,
        "toggle" => toggle::invoke(state, payload)?,
        "hover" => state.page.hover(&Locator::css(super::value::string_field(
            payload, "selector",
        )?))?,
        _ => unreachable!(),
    };
    if let Some(locator) = super::interact_focus::locator(action, payload)? {
        state.focused = Some(locator);
    }
    Ok(super::interact_value::report(report))
}
