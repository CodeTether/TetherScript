//! Parameter binding: the child scope a macro body renders against.
//!
//! The scope starts **empty** rather than cloning the caller's context, which is the one
//! thing this file exists to guarantee. Tera's macros are hermetic: a body sees only its
//! own parameters. A macro that reads a caller variable works by luck at its first call
//! site and renders a blank — or the wrong row — at its second. Building the scope the way
//! [`super::template_loop`] builds a loop scope, but from `HashMap::new()`, turns that
//! mistake into a named lookup error instead.
//!
//! Agreement checks live in [`super::template_macro_check`]; this file only assembles.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::template_macro::Macro;
use super::template_macro_arg::Arg;
use super::template_macro_check::{default_of, reject_unknown};
use super::template_macro_value::resolve;
use crate::value::Value;

/// Build the scope for one macro call.
///
/// # Arguments
///
/// * `def` — The macro definition whose parameters are being bound.
/// * `path` — Call path as written, for error messages.
/// * `args` — Supplied keyword arguments.
/// * `context` — Caller context, used only to evaluate argument expressions.
///
/// # Returns
///
/// A `Value::Map` holding exactly the macro's parameters and nothing else.
///
/// # Errors
///
/// Returns an error naming the parameter for a missing required argument, naming the
/// keyword for an argument that matches no parameter, or propagating an argument
/// expression that does not resolve.
pub(super) fn bind(
    def: &Macro<'_>,
    path: &str,
    args: &[Arg<'_>],
    context: &Value,
) -> Result<Value, String> {
    reject_unknown(def, path, args)?;
    let mut scope: HashMap<String, Value> = HashMap::new();
    for param in &def.params {
        let value = match args.iter().find(|arg| arg.name == param.name) {
            Some(arg) => resolve(arg.expression, context, path, param.name)?,
            None => default_of(def, path, param.name, param.default)?,
        };
        scope.insert(param.name.to_string(), value);
    }
    Ok(Value::Map(Rc::new(RefCell::new(scope))))
}
