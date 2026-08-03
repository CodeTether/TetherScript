//! Parameter binding: the child context a macro body renders against.
//!
//! # The one guarantee this file exists to make
//!
//! The child context starts **empty**, not as a clone of the caller's context. That is the
//! defining difference between `{% macro %}` and `{% include %}`: an include shares the
//! caller's scope, a macro does not. A macro body sees **only its parameters**.
//!
//! Why enforce it rather than allow the convenience? A body that reads an ambient caller
//! key works by luck at its first call site — where that key happens to exist — and renders
//! a blank, or worse the *wrong row*, at its second. The component then silently depends on
//! state its signature never declared, and the failure surfaces as wrong output on one page
//! rather than as an error anywhere. Starting from `HashMap::new()` converts that entire
//! class of bug into a named lookup error at the first call site.
//!
//! The caller's context is still read here, but only to evaluate the *argument
//! expressions*, which are call-site expressions and belong to the caller's scope. Their
//! resolved values are what cross the boundary; the context itself does not.
//!
//! Agreement checks live in [`crate::tmplmacro::check`]; this file only assembles.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::tmplmacro::args::Arg;
use crate::tmplmacro::argvalue::resolve;
use crate::tmplmacro::check::{default_of, reject_unknown};
use crate::tmplmacro::macros::MacroDef;
use crate::value::Value;

/// Build the child context for one macro call.
///
/// # Arguments
///
/// * `def` — The macro definition whose parameters are being bound.
/// * `path` — Call path as written, for error messages.
/// * `args` — Supplied keyword arguments.
/// * `caller` — Caller context, used **only** to evaluate argument expressions.
///
/// # Returns
///
/// A [`Value::Map`] holding exactly the macro's declared parameters and nothing else.
///
/// # Errors
///
/// Returns an error naming the parameter for a missing required argument, naming the
/// keyword for an argument matching no parameter, or propagating an argument expression
/// that does not resolve in `caller`.
///
/// # Panics
///
/// None.
///
/// # Examples
///
/// ```
/// use std::cell::RefCell;
/// use std::collections::HashMap;
/// use std::rc::Rc;
/// use tetherscript::tmplmacro::args::parse_args;
/// use tetherscript::tmplmacro::bind::bind;
/// use tetherscript::tmplmacro::macros::collect;
/// use tetherscript::value::Value;
///
/// let set = collect(r#"{% macro b(kind, size="sm") %}{% endmacro %}"#).unwrap();
/// let mut outer = HashMap::new();
/// outer.insert("secret".to_string(), Value::Int(1));
/// let caller = Value::Map(Rc::new(RefCell::new(outer)));
///
/// let args = parse_args(r#"kind="new""#, "b").unwrap();
/// let child = bind(&set["b"], "b", &args, &caller).unwrap();
/// let Value::Map(map) = child else { panic!("expected a map") };
/// let map = map.borrow();
/// assert_eq!(map.len(), 2);
/// assert!(map.contains_key("kind") && map.contains_key("size"));
/// // The caller's own keys are absent: a macro body is hermetic.
/// assert!(!map.contains_key("secret"));
/// ```
pub fn bind(
    def: &MacroDef,
    path: &str,
    args: &[Arg<'_>],
    caller: &Value,
) -> Result<Value, String> {
    reject_unknown(def, path, args)?;
    let mut scope: HashMap<String, Value> = HashMap::new();
    for param in &def.params {
        let value = match args.iter().find(|arg| arg.name == param.name.as_str()) {
            Some(arg) => resolve(arg.expression, caller, path, &param.name)?,
            None => default_of(def, path, &param.name, param.default.as_deref())?,
        };
        scope.insert(param.name.clone(), value);
    }
    Ok(Value::Map(Rc::new(RefCell::new(scope))))
}
