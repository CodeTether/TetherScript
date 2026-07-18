//! Replay of a captured request through the live native page runtime.

use crate::value::Value;

use super::super::state::HostState;

#[cfg(test)]
#[path = "network_replay_tests.rs"]
mod tests;

pub(super) fn invoke(state: &mut HostState, payload: &Value) -> Result<Value, String> {
    let contains = super::super::value::string_field(payload, "url_contains")?;
    let event = state
        .page
        .network_events()
        .iter()
        .rev()
        .find(|event| event.url.contains(&contains))
        .cloned()
        .ok_or_else(|| format!("browser.replay: no captured request contains `{contains}`"))?;
    let body = super::super::value::optional_string(payload, "body_patch")?;
    super::network_request::execute(state, "replay", &event.url, &event.method, body.as_deref())
}
