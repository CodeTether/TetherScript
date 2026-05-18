//! Live document-wide HTMLCollection queries.

use super::super::*;

pub(super) fn collection(handle: &DomHandle, pred: impl Fn(&Element) -> bool + 'static) -> JsValue {
    let root = handle.root.clone();
    let source = Rc::new(move || {
        let mut out = Vec::new();
        collect(&root.borrow(), &mut Vec::new(), &root, &pred, &mut out);
        out
    });
    live_collection_host::from_source(source, "HTMLCollection")
}

fn collect(
    node: &Node,
    path: &mut Vec<usize>,
    root: &Rc<RefCell<Node>>,
    pred: &impl Fn(&Element) -> bool,
    out: &mut Vec<DomHandle>,
) {
    let Node::Element(el) = node else {
        return;
    };
    if pred(el) {
        out.push(DomHandle {
            root: root.clone(),
            path: path.clone(),
        });
    }
    for (index, child) in el.children.iter().enumerate() {
        path.push(index);
        collect(child, path, root, pred, out);
        path.pop();
    }
}
