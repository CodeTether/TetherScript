//! Native multi-page tab actions.

use crate::value::Value;

use super::super::state::HostState;

#[path = "tabs_close.rs"]
mod close;
#[path = "tabs_new.rs"]
mod new;
#[path = "tabs_select.rs"]
mod select;
#[path = "tabs_value.rs"]
mod value;

#[cfg(test)]
#[path = "tabs_tests.rs"]
mod tests;

pub(super) fn invoke(
    state: &mut HostState,
    action: &str,
    payload: &Value,
) -> Result<Value, String> {
    match action {
        "tabs" => {}
        "tabs_new" => new::open(state, &super::super::value::string_field(payload, "url")?)?,
        "tabs_select" => select::activate(state, index(payload, action)?)?,
        "tabs_close" => close::close(state, index(payload, action)?)?,
        _ => unreachable!(),
    }
    Ok(value::list(state))
}

fn index(payload: &Value, action: &str) -> Result<usize, String> {
    let Some(index) = super::super::value::optional_int(payload, "index")? else {
        return Err(format!("browser.{action}: missing `index`"));
    };
    usize::try_from(index).map_err(|_| format!("browser.{action}: index must be non-negative"))
}
