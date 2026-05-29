//! Dispatch from tetherscript computer methods to CodeTether actions.

use crate::value::Value;

use super::call::ComputerCall;

#[path = "map_action.rs"]
mod map_action;
#[path = "map_raw.rs"]
mod map_raw;

pub(crate) fn prepare(method: &str, args: &[Value]) -> Result<ComputerCall, String> {
    match method {
        "raw" => map_raw::prepare(args),
        method if is_method(method) => map_action::prepare(method, args),
        _ => Err(format!("computer: no method `{}`", method)),
    }
}

pub(super) fn scoped(action: &str, payload: Value) -> Result<ComputerCall, String> {
    let scope = super::actions::scope_for_action(action)
        .ok_or_else(|| format!("computer: unknown action `{}`", action))?;
    Ok(ComputerCall::new(action, scope, payload))
}

fn is_method(method: &str) -> bool {
    matches!(
        method,
        "status"
            | "list_apps"
            | "snapshot"
            | "window_snapshot"
            | "request_app"
            | "bring_to_front"
            | "click"
            | "right_click"
            | "double_click"
            | "drag"
            | "mouse_down"
            | "mouse_move"
            | "mouse_up"
            | "type_text"
            | "press_key"
            | "scroll"
            | "focus_viewport"
            | "blender_select_frame"
            | "wait_ms"
            | "stop"
    )
}
