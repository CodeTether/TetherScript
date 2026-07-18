use super::*;

pub(in crate::browser_js) fn focused() -> Option<DomHandle> {
    FOCUSED_ELEMENT.with(|focused| {
        focused
            .borrow()
            .as_ref()
            .and_then(|key| handle_by_event_key(key))
    })
}

pub(in crate::browser_js) fn focused_editable() -> Option<DomHandle> {
    let handle = focused()?;
    let Node::Element(element) = handle.node()? else {
        return None;
    };
    (is_text_control(&element) || is_contenteditable(&element)).then_some(handle)
}

pub(in crate::browser_js) fn text() -> String {
    props::selection_text()
}

pub(in crate::browser_js) fn select_all(handle: &DomHandle) -> bool {
    let Some(Node::Element(element)) = handle.node() else {
        return false;
    };
    if is_text_control(&element) {
        set_selection_for_handle(handle, 0, handle.input_value().chars().count());
    } else if is_contenteditable(&element) {
        registry::set_selection(Some(state::RangeState::select_contents(handle)));
    } else {
        return false;
    }
    true
}

fn is_text_control(element: &Element) -> bool {
    if element.tag == "textarea" {
        return true;
    }
    let kind = element
        .attrs
        .get("type")
        .map(|kind| kind.to_ascii_lowercase())
        .unwrap_or_else(|| "text".into());
    element.tag == "input"
        && matches!(
            kind.as_str(),
            "text" | "search" | "tel" | "url" | "email" | "password"
        )
}
