use super::*;

pub(in crate::browser_js) fn install_window(window: &mut HashMap<String, JsValue>) {
    parser::install(window);
    serializer::install(window);
    traversal::install_window(window);
}

pub(in crate::browser_js) fn install_node(
    obj: &mut HashMap<String, JsValue>,
    handle: &DomHandle,
    node: &Node,
) {
    document::install(obj, handle, node);
    template::install(obj, node);
    dialog::install(obj, handle, node);
    popover::install(obj, handle, node);
    file_input::install(obj, node);
    convenience::install(obj, handle, node);
    traversal::install_node(obj, handle, node);
}

pub(in crate::browser_js) fn install_live_node(
    obj: &Rc<RefCell<HashMap<String, JsValue>>>,
    handle: &DomHandle,
    node: &Node,
) {
    convenience::scroll_metrics::install_live(obj, handle, node);
    form_validation::install(obj, handle, node);
}
