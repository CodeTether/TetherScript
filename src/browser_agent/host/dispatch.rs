//! Browser action-envelope dispatch for one native page session.

use crate::value::Value;

use super::state::HostState;

#[path = "diagnose.rs"]
mod diagnose;
#[path = "indexed_db_summary.rs"]
mod indexed_db_summary;
#[path = "network_replay.rs"]
mod network_replay;
#[path = "network_request.rs"]
mod network_request;

pub(super) fn invoke(state: &mut HostState, payload: &Value) -> (Result<Value, String>, bool) {
    let action = match super::value::string_field(payload, "action") {
        Ok(action) => action,
        Err(error) => return (Err(error), false),
    };
    let stop = action == "stop";
    let result = match action.as_str() {
        "health" | "detect" | "start" | "stop" | "goto" | "reload" | "back" | "forward"
        | "tabs" | "tabs_new" | "tabs_select" | "tabs_close" => {
            super::nav::invoke(state, &action, payload)
        }
        "snapshot" => Ok(super::snapshot::value(&state.page)),
        "text" | "html" | "eval" | "network_log" => super::query::invoke(state, &action, payload),
        "fetch" | "axios" | "xhr" => network_request::invoke(state, &action, payload),
        "replay" => network_replay::invoke(state, payload),
        "diagnose" => diagnose::invoke(state, payload),
        "visual_compare" => super::screenshot::visual_compare::invoke(state, payload),
        "indexed_db_summary" => indexed_db_summary::invoke(state),
        "wait" => super::wait::invoke(state, payload),
        "click" | "click_text" | "fill" | "type" | "upload" | "toggle" | "mouse_click"
        | "hover" => super::interact::invoke(state, &action, payload),
        "focus" | "blur" => super::focus::invoke(state, &action, payload),
        "press" | "keyboard_press" | "keyboard_type" => {
            super::keyboard::invoke(state, &action, payload)
        }
        "scroll" => super::scroll::invoke(state, payload),
        "screenshot" => super::screenshot::invoke(state, payload),
        _ => Err(format!(
            "browser host: native action `{}` is not implemented",
            action
        )),
    };
    (result, stop)
}
