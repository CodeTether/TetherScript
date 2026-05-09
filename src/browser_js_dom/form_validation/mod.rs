use super::*;

mod check;
mod control_methods;
mod controls;
mod custom;
mod dispatch;
mod elements_methods;
mod elements_object;
mod elements_sync;
mod form;
mod form_methods;
mod form_props;
mod listed;
mod message;
mod names;
mod object;
mod refresh;
mod select;
#[cfg(test)]
mod tests;
mod types;
mod value_setter;
mod values;

pub(super) fn install(
    obj: &Rc<RefCell<HashMap<String, JsValue>>>,
    handle: &DomHandle,
    node: &Node,
) {
    dispatch::install(obj, handle, node);
}

pub(crate) fn reset() {
    custom::reset();
    select::reset();
}
