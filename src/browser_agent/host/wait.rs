//! Deterministic native browser wait predicates.

use crate::value::Value;

use super::state::HostState;

pub(super) fn invoke(state: &HostState, payload: &Value) -> Result<Value, String> {
    if let Some(selector) = super::value::optional_string(payload, "selector")? {
        return super::query::text(state, Some(selector)).map(|_| Value::Bool(true));
    }
    if let Some(expected) = super::value::optional_string(payload, "text")? {
        let found = super::snapshot::document_text(&state.page).contains(&expected);
        return Ok(Value::Bool(found));
    }
    if let Some(expected) = super::value::optional_string(payload, "url_contains")? {
        return Ok(Value::Bool(state.page.session.url.contains(&expected)));
    }
    Err("browser.wait: expected selector, text, or url_contains".into())
}
