//! Registration of the rendering built-ins.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Env;

use super::template_filter::{str_arg, wrap};
use super::template_install::native;
use super::template_render::{render, render_inherited, render_lenient};

/// Define `template_render`, `template_render_raw`, `template_render_inherited`.
pub(super) fn install(env: &Rc<RefCell<Env>>) {
    let mut bindings = env.borrow_mut();
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
    bindings.define(
        "template_render_inherited",
        native("template_render_inherited", 3, |args| {
            let template = str_arg(&args[0], "template_render_inherited: template")?;
            Ok(wrap(render_inherited(&template, &args[1], &args[2], true)))
        }),
        false,
    );
    bindings.define(
        "template_render_lenient",
        // Same as `template_render_inherited`, except an unknown key renders as empty. For a view
        // tree written against Tera, whose own default is lenient: one key a port has no equivalent
        // for must not take a whole page down.
        native("template_render_lenient", 3, |args| {
            let template = str_arg(&args[0], "template_render_lenient: template")?;
            Ok(wrap(render_lenient(&template, &args[1], &args[2])))
        }),
        false,
    );
}
