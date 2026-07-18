//! Native locator-backed replacement fill action.

use crate::browser_agent::{ActionReport, Locator};
use crate::value::Value;

use super::super::state::HostState;

#[cfg(test)]
#[path = "fill_native_tests.rs"]
mod native_tests;

pub(super) fn invoke(state: &mut HostState, payload: &Value) -> Result<ActionReport, String> {
    state.page.fill(
        &Locator::css(super::super::value::string_field(payload, "selector")?),
        &super::super::value::string_field(payload, "value")?,
    )
}
