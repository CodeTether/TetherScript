//! Native locator-backed click actions.

use crate::browser_agent::{ActionReport, Locator};
use crate::value::Value;

use super::super::state::HostState;

#[cfg(test)]
#[path = "click_text_tests.rs"]
mod tests;

pub(super) fn invoke(
    state: &mut HostState,
    action: &str,
    payload: &Value,
) -> Result<ActionReport, String> {
    let locator = if action == "click" {
        Locator::css(super::super::value::string_field(payload, "selector")?)
    } else {
        Locator::text_exact(super::super::value::string_field(payload, "text")?)
    };
    state.page.click(&locator)
}
