//! Agent-initiated requests through the native page network runtime.

use crate::value::Value;

use super::super::state::HostState;

#[path = "network_request_result.rs"]
mod result;
#[path = "network_request_script.rs"]
mod script;
#[cfg(test)]
#[path = "network_request_tests.rs"]
mod tests;

pub(super) fn invoke(
    state: &mut HostState,
    action: &str,
    payload: &Value,
) -> Result<Value, String> {
    let url = super::super::value::string_field(payload, "url")?;
    let method =
        super::super::value::optional_string(payload, "method")?.unwrap_or_else(|| "GET".into());
    let body = super::super::value::optional_string(payload, "body")?;
    execute(state, action, &url, &method, body.as_deref())
}

pub(super) fn execute(
    state: &mut HostState,
    action: &str,
    url: &str,
    method: &str,
    body: Option<&str>,
) -> Result<Value, String> {
    state
        .page
        .eval_js(&script::source(action, url, method, body)?)?;
    let response = state.page.eval_js("window.__tetherscriptHostResponse")?;
    result::validate(crate::js::js_to_tether(&response), action)
}
