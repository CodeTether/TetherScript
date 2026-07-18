//! Native locator-backed click, fill, type, and hover actions.

use crate::browser_agent::{ActionReport, Locator};
use crate::value::Value;

use super::state::HostState;

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
        "fill" | "type" => fill(state, payload, action)?,
        "hover" => state.page.hover(&Locator::css(super::value::string_field(
            payload, "selector",
        )?))?,
        _ => unreachable!(),
    };
    Ok(super::interact_value::report(report))
}

fn fill(state: &mut HostState, payload: &Value, action: &str) -> Result<ActionReport, String> {
    let key = if action == "fill" { "value" } else { "text" };
    state.page.fill(
        &Locator::css(super::value::string_field(payload, "selector")?),
        &super::value::string_field(payload, key)?,
    )
}
