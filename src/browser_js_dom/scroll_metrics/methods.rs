use super::*;

#[path = "method_absolute.rs"]
mod method_absolute;
#[path = "method_into.rs"]
mod method_into;
#[path = "method_relative.rs"]
mod method_relative;

pub(super) fn install(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    handle: &DomHandle,
) {
    method_absolute::install(object, handle);
    method_relative::install(object, handle);
    method_into::install(object, handle);
}
