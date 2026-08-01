//! Registration of the template built-ins.
//!
//! Split from `template.rs` so the entry point keeps its documentation and the
//! binding list stays readable.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::{Env, Value};

use super::template_escape::{escape, escape_attr};
use super::template_render::render;
use super::{native, str_arg, wrap};

/// Define `html_escape`, `html_attr`, `template_render`, `template_render_raw`.
pub(super) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
    bindings.define(
        "html_escape",
        native("html_escape", 1, |args| {
            let text = str_arg(&args[0], "html_escape: text")?;
            Ok(Value::Str(Rc::new(escape(&text))))
        }),
        false,
    );
    bindings.define(
        "html_attr",
        native("html_attr", 1, |args| {
            let text = str_arg(&args[0], "html_attr: text")?;
            Ok(Value::Str(Rc::new(escape_attr(&text))))
        }),
        false,
    );
    bindings.define(
        "template_render",
        native("template_render", 2, |args| {
            let template = str_arg(&args[0], "template_render: template")?;
            Ok(wrap(render(&template, &args[1], true)))
        }),
        false,
    );
    bindings.define(
        "template_render_raw",
        native("template_render_raw", 2, |args| {
            let template = str_arg(&args[0], "template_render_raw: template")?;
            Ok(wrap(render(&template, &args[1], false)))
        }),
        false,
    );
}
