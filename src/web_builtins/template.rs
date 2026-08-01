//! Dependency-free HTML template built-ins.
//!
//! the reference application renders every page through Tera, but tetherscript's Tera support
//! sits behind the optional `tera` feature, so the default zero-dependency build
//! cannot render a page at all. This group provides a small always-available
//! renderer so the port can serve HTML without a dependency. It is deliberately
//! not a Tera clone: there are no filters, loops, conditionals, or inheritance.
//! Use `tera_render` when the `tera` feature is enabled and those are needed.
//!
//! # Script surface
//!
//! | Builtin | Returns |
//! |---|---|
//! | `html_escape(text)` | escaped str |
//! | `html_attr(text)` | str safe inside a quoted attribute |
//! | `template_render(template, context)` | `Result` of rendered str, escaping by default |
//! | `template_render_raw(template, context)` | `Result` of rendered str, no escaping |
//!
//! # Security
//!
//! **`{{ name }}` escapes; `{{{ name }}}` does not.** Escaping by default is the
//! entire point: a renderer that interpolates untrusted values verbatim is an XSS
//! vector, so opting out has to be explicit and visible in the template. Reach for
//! the triple-brace form only for markup you generated yourself.
//!
//! An unknown key is a named `Err`, never an empty string, so a typo cannot
//! quietly blank part of a page. `template_render_raw` escapes nothing and is
//! intended for non-HTML output such as plain text or CSV.
//!
//! # Examples
//!
//! ```tether
//! let ctx = map()
//! ctx.title = "Hello & welcome"
//! // Renders: <h1>Hello &amp; welcome</h1>
//! println(template_render("<h1>\{\{ title \}\}</h1>", ctx).unwrap())
//! ```

use std::cell::RefCell;
use std::rc::Rc;

use crate::system::result_value;
use crate::value::{Env, Value};

use super::super::pure_native;

#[path = "template_context.rs"]
pub(super) mod template_context;
#[path = "template_escape.rs"]
pub(super) mod template_escape;
#[path = "template_install.rs"]
pub(super) mod template_install;
#[path = "template_render.rs"]
pub(super) mod template_render;

/// Register this group's built-ins.
pub(crate) fn install(env: &Rc<RefCell<Env>>) {
    template_install::install(env);
}

/// Coerce a built-in argument to a string, naming the parameter on mismatch.
pub(super) fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}

/// Wrap a fallible render as a tetherscript `Result`.
pub(super) fn wrap(result: Result<String, String>) -> Value {
    result_value(result.map(|text| Value::Str(Rc::new(text))))
}

/// Build a pure native from a name, arity, and body.
pub(super) fn native<F>(name: &str, arity: usize, func: F) -> Value
where
    F: Fn(&[Value]) -> Result<Value, String> + 'static,
{
    pure_native(name, Some(arity), func)
}
