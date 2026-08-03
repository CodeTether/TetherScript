//! Registration of the escaping built-ins.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::{Env, Value};

use super::template_escape::{escape, escape_attr};
use super::template_filter::str_arg;
use super::template_install::native;

/// Define `html_escape` and `html_attr`.
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
}
