//! CodeTether computer-use action classification for scopes.

pub(crate) fn scope_for_action(action: &str) -> Option<&'static str> {
    match action {
        "status" | "list_apps" | "request_app" => Some("computer.apps"),
        "snapshot" => Some("computer.snapshot"),
        "window_snapshot" | "bring_to_front" | "focus_viewport" => Some("computer.window_snapshot"),
        "click"
        | "right_click"
        | "double_click"
        | "drag"
        | "mouse_down"
        | "mouse_move"
        | "mouse_up"
        | "blender_select_frame" => Some("computer.click"),
        "type_text" => Some("computer.type"),
        "press_key" => Some("computer.key"),
        "scroll" => Some("computer.scroll"),
        "wait_ms" | "stop" => Some("computer.apps"),
        _ => None,
    }
}
