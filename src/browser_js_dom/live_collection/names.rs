//! Named access for live HTML collections.

use super::*;

pub(super) fn install(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    parent: DomHandle,
    kind: &'static str,
) {
    object.borrow_mut().insert(
        "namedItem".into(),
        native(&format!("{kind}.namedItem"), Some(1), move |args| {
            let name = args.first().unwrap_or(&JsValue::Undefined).display();
            Ok(super::handles(&parent)
                .into_iter()
                .find(|handle| matches_name(handle, &name))
                .map(node_object)
                .unwrap_or(JsValue::Null))
        }),
    );
}

fn matches_name(handle: &DomHandle, name: &str) -> bool {
    names(handle).iter().any(|item| item == name)
}

fn names(handle: &DomHandle) -> Vec<String> {
    let Some(Node::Element(el)) = handle.node() else {
        return Vec::new();
    };
    ["id", "name"]
        .iter()
        .filter_map(|attr| el.attrs.get(*attr))
        .filter(|name| !name.is_empty())
        .cloned()
        .collect()
}
