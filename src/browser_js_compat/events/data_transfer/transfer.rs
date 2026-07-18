use super::*;

pub(in crate::browser_js) struct Transfer {
    value: JsValue,
    strings: model::SharedStrings,
}

impl Transfer {
    pub(in crate::browser_js) fn new(text: &str) -> Self {
        let (value, strings, types) = object::create_state();
        model::set_string(&mut strings.borrow_mut(), "text/plain".into(), text.into());
        model::sync_types(&types, &strings.borrow());
        Self { value, strings }
    }

    pub(in crate::browser_js) fn value(&self) -> JsValue {
        self.value.clone()
    }

    pub(in crate::browser_js) fn text(&self) -> String {
        model::get_string(&self.strings.borrow(), "text/plain")
    }
}
