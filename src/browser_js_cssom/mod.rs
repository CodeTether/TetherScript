mod build;
mod collect;
mod decl_object;
mod index;
mod list_object;
mod model;
mod mutation;
mod parse_decl;
mod parse_rule;
mod rule_object;
mod rules;
mod sheet_object;
mod state;
mod sync;

use std::rc::Rc;

use crate::browser::Document;
use crate::js::JsValue;

use state::Cssom;

pub(super) fn install_document(document: &JsValue, model: &Document, css: String) {
    let cssom = Cssom::new(model, css, Rc::new(layout_update));
    if let JsValue::Object(obj) = document {
        obj.borrow_mut()
            .insert("styleSheets".into(), list_object::object(&cssom));
    }
}

fn layout_update(source: &str) {
    super::LAYOUT_CSS.with(|css| *css.borrow_mut() = source.to_string());
}

fn native(
    name: &str,
    arity: Option<usize>,
    func: impl Fn(&[JsValue]) -> Result<JsValue, String> + 'static,
) -> JsValue {
    super::native(name, arity, func)
}
