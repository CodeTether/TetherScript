//! Named access for live HTML collections.

use super::*;

pub(super) fn install(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    source: Source,
    kind: &'static str,
) {
    let named_source = source.clone();
    object.borrow_mut().insert(
        "namedItem".into(),
        native(&format!("{kind}.namedItem"), Some(1), move |args| {
            let name = args.first().unwrap_or(&JsValue::Undefined).display();
            Ok(named(&named_source, &name, JsValue::Null))
        }),
    );
    object.borrow_mut().insert(
        "__get:*".into(),
        native(&format!("{kind}.namedProperty"), Some(1), move |args| {
            let name = args.first().unwrap_or(&JsValue::Undefined).display();
            Ok(named(&source, &name, JsValue::Undefined))
        }),
    );
}

fn named(source: &Source, name: &str, miss: JsValue) -> JsValue {
    super::handles(source)
        .into_iter()
        .find(|handle| matches_name(handle, name))
        .map(node_object)
        .unwrap_or(miss)
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
