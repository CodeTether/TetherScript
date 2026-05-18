//! Indexed live collection accessors.

use super::*;

const MAX_INDEX: usize = 1024;

pub(super) fn install(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    source: Source,
    kind: &'static str,
) {
    let len_source = source.clone();
    object.borrow_mut().insert(
        "__get:length".into(),
        native(&format!("{kind}.length"), Some(0), move |_| {
            Ok(JsValue::Number(super::handles(&len_source).len() as f64))
        }),
    );
    for index in 0..MAX_INDEX {
        install_index(object, source.clone(), kind, index);
    }
}

fn install_index(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    source: Source,
    kind: &'static str,
    index: usize,
) {
    object.borrow_mut().insert(
        format!("__get:{index}"),
        native(&format!("{kind}.{index}"), Some(0), move |_| {
            Ok(super::at(&source, index))
        }),
    );
}
