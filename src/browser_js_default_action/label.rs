//! Label activation default action.

use super::super::*;

pub(super) fn activate(handle: &DomHandle) -> Result<bool, String> {
    if let Some(target) = explicit_target(handle).or_else(|| descendant_target(handle)) {
        target.dispatch_event(JsValue::String("click".into()))?;
    }
    Ok(true)
}

fn explicit_target(handle: &DomHandle) -> Option<DomHandle> {
    let id = match handle.node()? {
        Node::Element(el) => el.attrs.get("for")?.clone(),
        Node::Text(_) => return None,
    };
    find_by_id(&handle.root, &id).map(|path| DomHandle {
        root: handle.root.clone(),
        path,
    })
}

fn descendant_target(handle: &DomHandle) -> Option<DomHandle> {
    let Node::Element(el) = handle.node()? else {
        return None;
    };
    for (index, child) in el.children.iter().enumerate() {
        let mut path = handle.path.clone();
        path.push(index);
        let child_handle = DomHandle {
            root: handle.root.clone(),
            path,
        };
        if labelable(child) {
            return Some(child_handle);
        }
        if let Some(found) = descendant_target(&child_handle) {
            return Some(found);
        }
    }
    None
}

fn labelable(node: &Node) -> bool {
    let Node::Element(el) = node else {
        return false;
    };
    matches!(
        el.tag.as_str(),
        "button" | "input" | "meter" | "output" | "progress" | "select" | "textarea"
    ) && !el
        .attrs
        .get("type")
        .is_some_and(|ty| ty.eq_ignore_ascii_case("hidden"))
}
