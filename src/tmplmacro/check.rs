//! Argument/parameter agreement checks for one macro call.
//!
//! Split from [`crate::tmplmacro::bind`], which assembles the scope; this file decides
//! whether a call is well-formed enough to have one. Both checks exist because a template
//! error that renders anyway is worse than one that stops:
//!
//! * A **typo'd keyword** silently dropped ships a subtly wrong page — the `cfg` a
//!   component needs written as `cfgs` would render an empty component with no diagnostic.
//! * A **missing required argument** silently niled ships a blank where content belongs.
//!
//! So an unknown keyword is rejected, and a parameter with no default and no argument is
//! rejected. A parameter *with* a default is filled from that default.

use crate::tmplmacro::args::Arg;
use crate::tmplmacro::literal::literal_of;
use crate::tmplmacro::macros::MacroDef;
use crate::tmplmacro::report::signature;
use crate::value::Value;

/// Reject any argument that names no declared parameter.
///
/// # Arguments
///
/// * `def` — The macro being called.
/// * `path` — Call path as written, for the message.
/// * `args` — Supplied keyword arguments.
///
/// # Returns
///
/// `Ok(())` when every argument matches a parameter.
///
/// # Errors
///
/// Returns an error naming the offending keyword, the macro, and the macro's signature.
///
/// # Examples
///
/// ```
/// use tetherscript::tmplmacro::args::parse_args;
/// use tetherscript::tmplmacro::check::reject_unknown;
/// use tetherscript::tmplmacro::macros::collect;
///
/// let set = collect("{% macro badge(kind) %}{% endmacro %}").unwrap();
/// let args = parse_args("kinds=1", "badge").unwrap();
/// assert!(reject_unknown(&set["badge"], "badge", &args).is_err());
/// ```
pub fn reject_unknown(def: &MacroDef, path: &str, args: &[Arg<'_>]) -> Result<(), String> {
    for arg in args {
        if !def.params.iter().any(|param| param.name.as_str() == arg.name) {
            return Err(format!(
                "template: macro `{path}` has no parameter `{}`; signature is {}",
                arg.name,
                signature(def)
            ));
        }
    }
    Ok(())
}

/// Value for a parameter the call omitted.
///
/// # Arguments
///
/// * `def` — The macro being called, for its signature in the message.
/// * `path` — Call path as written.
/// * `name` — Omitted parameter's name.
/// * `default` — Its raw default literal, or `None` when required.
///
/// # Returns
///
/// The converted default value.
///
/// # Errors
///
/// Returns an error naming the parameter and the signature when there is no default.
///
/// # Examples
///
/// ```
/// use tetherscript::tmplmacro::check::default_of;
/// use tetherscript::tmplmacro::macros::collect;
///
/// let set = collect(r#"{% macro b(k, s="sm") %}{% endmacro %}"#).unwrap();
/// let def = &set["b"];
/// assert!(default_of(def, "b", "s", Some("\"sm\"")).is_ok());
/// assert!(default_of(def, "b", "k", None).is_err());
/// ```
pub fn default_of(
    def: &MacroDef,
    path: &str,
    name: &str,
    default: Option<&str>,
) -> Result<Value, String> {
    match default {
        Some(literal) => Ok(literal_of(literal)),
        None => Err(format!(
            "template: macro `{path}` requires argument `{name}`; signature is {}",
            signature(def)
        )),
    }
}
