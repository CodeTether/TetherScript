use super::*;

pub(super) fn single(root: &Rc<RefCell<Node>>, x: i64, y: i64) -> JsValue {
    if !inside_viewport(x, y) {
        return JsValue::Null;
    }
    hit_collect::hits(root, x, y)
        .into_iter()
        .next()
        .map(|hit| node_for(root, hit.path))
        .unwrap_or(JsValue::Null)
}

pub(super) fn all(root: &Rc<RefCell<Node>>, x: i64, y: i64) -> JsValue {
    if !inside_viewport(x, y) {
        return JsValue::Array(Rc::new(RefCell::new(Vec::new())));
    }
    let nodes = hit_collect::hits(root, x, y)
        .into_iter()
        .map(|hit| node_for(root, hit.path))
        .collect();
    JsValue::Array(Rc::new(RefCell::new(nodes)))
}

fn node_for(root: &Rc<RefCell<Node>>, path: Vec<usize>) -> JsValue {
    node_object(DomHandle {
        root: root.clone(),
        path,
    })
}

fn inside_viewport(x: i64, y: i64) -> bool {
    x >= 0
        && y >= 0
        && x < constants::DEFAULT_VIEWPORT_WIDTH
        && y < constants::DEFAULT_VIEWPORT_HEIGHT
}
