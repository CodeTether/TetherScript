//! Native checkbox and radio toggle action.

use crate::browser_agent::{resolve, ActionReport, Locator};
use crate::value::Value;

use super::super::state::HostState;

#[cfg(test)]
#[path = "toggle_tests.rs"]
mod tests;

pub(super) fn invoke(state: &mut HostState, payload: &Value) -> Result<ActionReport, String> {
    let locator = Locator::css(super::super::value::string_field(payload, "selector")?);
    let resolved = resolve::resolve(&state.page.session, state.page.viewport_width, &locator)?;
    let mut report = if resolved.dom.element.attrs.contains_key("checked") {
        state.page.uncheck(&locator)?
    } else {
        state.page.check(&locator)?
    };
    report.action = "toggle".into();
    Ok(report)
}
