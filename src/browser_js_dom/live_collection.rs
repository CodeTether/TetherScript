//! Live DOM child collection objects.

use super::*;

#[path = "live_collection/indices.rs"]
mod indices;
#[path = "live_collection/items.rs"]
mod items;
#[path = "live_collection/names.rs"]
mod names;

pub(super) fn children(parent: &DomHandle, kind: &'static str) -> JsValue {
    let object = Rc::new(RefCell::new(HashMap::new()));
    indices::install(&object, parent.clone(), kind);
    items::install(&object, parent.clone(), kind);
    if kind == "HTMLCollection" {
        names::install(&object, parent.clone(), kind);
    }
    JsValue::Object(object)
}

pub(super) fn handles(parent: &DomHandle) -> Vec<DomHandle> {
    let Some(Node::Element(el)) = parent.node() else {
        return Vec::new();
    };
    (0..el.children.len())
        .map(|index| {
            let mut path = parent.path.clone();
            path.push(index);
            DomHandle {
                root: parent.root.clone(),
                path,
            }
        })
        .collect()
}

pub(super) fn at(parent: &DomHandle, index: usize) -> JsValue {
    handles(parent)
        .get(index)
        .cloned()
        .map(node_object)
        .unwrap_or(JsValue::Null)
}
