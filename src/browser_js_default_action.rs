//! DOM click default actions.

use super::*;

#[path = "browser_js_default_action/anchor.rs"]
mod anchor;
#[path = "browser_js_default_action/form.rs"]
mod form;
#[path = "browser_js_default_action/label.rs"]
mod label;

pub(super) fn reset() {
    anchor::reset();
}

pub(super) fn register_location(
    root: &Rc<RefCell<Node>>,
    location: Rc<RefCell<HashMap<String, JsValue>>>,
) {
    anchor::register_location(root, location);
}

pub(super) fn run(handle: &DomHandle, event_type: &str, event: &JsValue) -> Result<bool, String> {
    if event_type != "click" {
        return Ok(true);
    }
    let Some(Node::Element(el)) = handle.node() else {
        return Ok(true);
    };
    match el.tag.as_str() {
        "input" => form::input(handle, &el),
        "button" => form::button(handle, &el),
        "label" => label::activate(handle),
        "a" | "area" => anchor::navigate(handle, &el, event),
        _ => Ok(true),
    }
}
