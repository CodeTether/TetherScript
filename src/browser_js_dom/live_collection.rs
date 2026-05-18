//! Live DOM child collection objects.

use super::*;

#[path = "live_collection/indices.rs"]
mod indices;
#[path = "live_collection/items.rs"]
mod items;
#[path = "live_collection/names.rs"]
mod names;

pub(super) type Source = Rc<dyn Fn() -> Vec<DomHandle>>;

pub(super) fn children(parent: &DomHandle, kind: &'static str) -> JsValue {
    let parent = parent.clone();
    from_source(Rc::new(move || child_handles(&parent)), kind)
}

pub(super) fn from_source(source: Source, kind: &'static str) -> JsValue {
    let object = Rc::new(RefCell::new(HashMap::new()));
    indices::install(&object, source.clone(), kind);
    items::install(&object, source.clone(), kind);
    if kind == "HTMLCollection" {
        names::install(&object, source, kind);
    }
    JsValue::Object(object)
}

pub(super) fn handles(source: &Source) -> Vec<DomHandle> {
    source()
}

pub(super) fn child_handles(parent: &DomHandle) -> Vec<DomHandle> {
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

pub(super) fn at(source: &Source, index: usize) -> JsValue {
    handles(source)
        .get(index)
        .cloned()
        .map(node_object)
        .unwrap_or(JsValue::Null)
}
