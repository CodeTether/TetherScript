//! Native viewport action envelope execution.

use crate::value::Value;

use super::state::HostState;

#[cfg(test)]
#[path = "viewport_tests.rs"]
mod tests;

pub(super) fn invoke(state: &mut HostState, payload: &Value) -> Result<Value, String> {
    let width = required(payload, "width")?;
    let height = required(payload, "height")?;
    state.page.set_viewport_size(width, height)?;
    Ok(super::value::map(vec![
        ("width", Value::Int(width)),
        ("height", Value::Int(height)),
    ]))
}

fn required(payload: &Value, name: &str) -> Result<i64, String> {
    super::value::optional_int(payload, name)?
        .ok_or_else(|| format!("browser.set_viewport: missing `{name}`"))
}
