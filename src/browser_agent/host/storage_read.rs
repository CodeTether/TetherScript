//! Native current-origin cookie and DOM-storage observations.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

use super::super::super::state::HostState;

#[cfg(test)]
#[path = "storage_read_tests.rs"]
mod tests;

pub(super) fn invoke(state: &mut HostState, action: &str) -> Result<Value, String> {
    state.page.sync_context_state_into_session();
    let origin = crate::browser_cookie::storage_origin(&state.page.session.url);
    match action {
        "cookies" => Ok(cookies(state)),
        "local_storage" => Ok(entries(state.page.session.local_storage.get(&origin))),
        "session_storage" => Ok(entries(state.page.session.session_storage.get(&origin))),
        _ => unreachable!(),
    }
}

fn cookies(state: &HostState) -> Value {
    let text = crate::browser_cookie::document_cookie_pairs(
        &state.page.session.cookies,
        &state.page.session.url,
    )
    .into_iter()
    .map(|(name, value)| format!("{name}={value}"))
    .collect::<Vec<_>>()
    .join("; ");
    super::super::super::value::string(text)
}

fn entries(values: Option<&HashMap<String, String>>) -> Value {
    let values = values.cloned().unwrap_or_default();
    Value::Map(Rc::new(RefCell::new(
        values
            .into_iter()
            .map(|(key, value)| (key, super::super::super::value::string(value)))
            .collect(),
    )))
}
