//! Captured native page network-event queries.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Value;

use super::super::state::HostState;

#[path = "network_log_filter.rs"]
mod filter;
#[cfg(test)]
#[path = "network_log_tests.rs"]
mod tests;
#[path = "network_log_value.rs"]
mod value;
#[path = "network_log_wait.rs"]
mod wait;

pub(super) fn invoke(state: &mut HostState, payload: &Value) -> Result<Value, String> {
    wait::until(state, payload)?;
    let contains = super::super::value::optional_string(payload, "url_contains")?;
    let limit = super::super::value::optional_int(payload, "limit")?
        .map(|value| {
            usize::try_from(value).map_err(|_| "browser.network_log: limit must be non-negative")
        })
        .transpose()?;
    let failed_only = filter::failed_only(payload)?;
    let mut values = state
        .page
        .network_events()
        .iter()
        .filter(|event| !failed_only || filter::failed(event))
        .filter(|event| {
            contains
                .as_ref()
                .is_none_or(|part| event.url.contains(part))
        })
        .map(value::from_event)
        .collect::<Vec<_>>();
    if let Some(limit) = limit {
        values.truncate(limit);
    }
    Ok(Value::List(Rc::new(RefCell::new(values))))
}
