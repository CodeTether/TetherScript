//! browserctl action classification for scopes and approval.

pub(crate) fn scope_for_action(action: &str) -> Option<&'static str> {
    match action {
        "goto" | "back" | "forward" | "reload" | "tabs" | "tabs_select" | "tabs_new"
        | "tabs_close" | "start" | "stop" => Some("browser.navigate"),
        "click" | "upload" | "fill" | "type" | "press" | "hover" | "focus" | "blur" | "scroll"
        | "click_text" | "fill_native" | "toggle" | "mouse_click" | "keyboard_type"
        | "keyboard_press" => Some("browser.interact"),
        "snapshot" | "text" | "html" | "wait" | "eval" => Some("browser.inspect.dom"),
        "network_log" | "diagnose" => Some("browser.inspect.network"),
        "fetch" | "axios" | "xhr" | "replay" => Some("browser.replay.network"),
        "screenshot" => Some("browser.screenshot"),
        "health" | "detect" => Some("browser.inspect.dom"),
        _ => None,
    }
}

pub(crate) fn is_mutating(action: &str) -> bool {
    match scope_for_action(action) {
        Some("browser.navigate") => action != "tabs",
        Some("browser.interact" | "browser.replay.network") => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn forward_is_a_mutating_navigation_action() {
        assert_eq!(super::scope_for_action("forward"), Some("browser.navigate"));
        assert!(super::is_mutating("forward"));
        assert!(!super::is_mutating("tabs"));
    }
}
