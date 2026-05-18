//! Live collection item and iteration methods.

use super::*;

pub(super) fn install(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    parent: DomHandle,
    kind: &'static str,
) {
    let item_parent = parent.clone();
    object.borrow_mut().insert(
        "item".into(),
        native(&format!("{kind}.item"), Some(1), move |args| {
            Ok(super::at(&item_parent, collection_index(args.first())))
        }),
    );
    let weak = Rc::downgrade(object);
    object.borrow_mut().insert(
        "forEach".into(),
        native(&format!("{kind}.forEach"), None, move |args| {
            iterate(args, &parent, &weak)
        }),
    );
}

fn iterate(
    args: &[JsValue],
    parent: &DomHandle,
    weak: &Weak<RefCell<HashMap<String, JsValue>>>,
) -> Result<JsValue, String> {
    let callback = args
        .first()
        .cloned()
        .ok_or_else(|| "collection.forEach: expected callback".to_string())?;
    let this_arg = args.get(1).cloned().unwrap_or(JsValue::Undefined);
    let collection = weak
        .upgrade()
        .map(JsValue::Object)
        .unwrap_or(JsValue::Undefined);
    for (index, handle) in super::handles(parent).into_iter().enumerate() {
        js::call_function_with_this(
            callback.clone(),
            this_arg.clone(),
            &[
                node_object(handle),
                JsValue::Number(index as f64),
                collection.clone(),
            ],
        )?;
    }
    Ok(JsValue::Undefined)
}
