//! Native page lifecycle and top-level history actions.

use crate::value::Value;

use super::state::HostState;

#[path = "tabs.rs"]
mod tabs;

pub(super) fn invoke(
    state: &mut HostState,
    action: &str,
    payload: &Value,
) -> Result<Value, String> {
    match action {
        "health" | "detect" => Ok(super::health::value(state)),
        "start" => {
            state.started = true;
            Ok(super::health::value(state))
        }
        "stop" => {
            state.started = false;
            Ok(super::health::value(state))
        }
        "goto" => super::nav_load::navigate(state, &super::value::string_field(payload, "url")?),
        "reload" => {
            let url = state.page.session.url.clone();
            super::nav_load::navigate(state, &url)
        }
        "back" => {
            state.focused = None;
            state.page.go_back();
            Ok(super::snapshot::value(&state.page))
        }
        "forward" => {
            state.focused = None;
            state.page.go_forward();
            Ok(super::snapshot::value(&state.page))
        }
        "tabs" | "tabs_new" | "tabs_select" | "tabs_close" => tabs::invoke(state, action, payload),
        _ => unreachable!(),
    }
}
