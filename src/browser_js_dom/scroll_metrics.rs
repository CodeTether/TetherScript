use super::super::*;

macro_rules! scroll_mod {
    ($path:literal, $name:ident) => {
        #[path = $path]
        mod $name;
    };
}

scroll_mod!("scroll_metrics/access.rs", access);
scroll_mod!("scroll_metrics/alignment.rs", alignment);
scroll_mod!("scroll_metrics/apply.rs", apply);
scroll_mod!("scroll_metrics/args.rs", args);
scroll_mod!("scroll_metrics_edges.rs", edges);
scroll_mod!("scroll_metrics/extent.rs", extent);
scroll_mod!("scroll_metrics/geometry.rs", geometry);
scroll_mod!("scroll_metrics/geometry_model.rs", geometry_model);
scroll_mod!("scroll_metrics/getters.rs", getters);
scroll_mod!("scroll_metrics/into_view.rs", into_view);
scroll_mod!("scroll_metrics/methods.rs", methods);
scroll_mod!("scroll_metrics/overflow.rs", overflow);
scroll_mod!("scroll_metrics/rect.rs", rect);
scroll_mod!("scroll_metrics/reveal.rs", reveal);
scroll_mod!("scroll_metrics/reveal_alignment.rs", reveal_alignment);
scroll_mod!("scroll_metrics/state.rs", state);
scroll_mod!("scroll_metrics/setters.rs", setters);
scroll_mod!("scroll_metrics/visibility.rs", visibility);

pub(in crate::browser_js) use access::{offset_for, point_visible, rekey, reset, scrolled_rect};

pub(in crate::browser_js) fn install_live(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    handle: &DomHandle,
    node: &Node,
) {
    if matches!(node, Node::Element(element) if !element.tag.starts_with('#')) {
        getters::install(object, handle);
        methods::install(object, handle);
    }
}

#[cfg(test)]
#[path = "scroll_metrics_tests.rs"]
mod scroll_metrics_tests;
#[cfg(test)]
#[path = "scroll_metrics/tests_into_view.rs"]
mod tests_into_view;
#[cfg(test)]
#[path = "scroll_metrics/tests_state.rs"]
mod tests_state;
