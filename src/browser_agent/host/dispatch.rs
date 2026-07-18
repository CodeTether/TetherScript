//! Browser action-envelope dispatch for one native page session.

use crate::value::Value;

use super::state::HostState;

pub(super) fn invoke(state: &mut HostState, payload: &Value) -> (Result<Value, String>, bool) {
    let action = match super::value::string_field(payload, "action") {
        Ok(action) => action,
        Err(error) => return (Err(error), false),
    };
    let stop = action == "stop";
    let result = match action.as_str() {
        "health" | "detect" | "start" | "stop" | "goto" | "reload" | "back" | "forward" => {
            super::nav::invoke(state, &action, payload)
        }
        "snapshot" => Ok(super::snapshot::value(&state.page)),
        "text" | "html" | "eval" => super::query::invoke(state, &action, payload),
        "wait" => super::wait::invoke(state, payload),
        "click" | "click_text" | "fill" | "type" | "upload" | "hover" => {
            super::interact::invoke(state, &action, payload)
        }
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
