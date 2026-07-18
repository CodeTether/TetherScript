//! Native viewport-coordinate mouse click action.

use crate::browser_agent::ActionReport;
use crate::value::Value;

use super::super::state::HostState;

pub(super) fn invoke(state: &mut HostState, payload: &Value) -> Result<ActionReport, String> {
    state
        .page
        .click_at(int_field(payload, "x")?, int_field(payload, "y")?)
}

fn int_field(payload: &Value, name: &str) -> Result<i64, String> {
    match super::super::value::field(payload, name)? {
        Value::Int(value) => Ok(value),
        value => Err(format!(
            "browser.mouse_click: `{name}` must be int, got {}",
            value.type_name()
        )),
    }
}
