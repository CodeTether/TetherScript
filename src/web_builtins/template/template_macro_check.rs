//! Argument/parameter agreement checks for a macro call.
//!
//! Split from [`super::template_macro_scope`], which builds the scope; this file decides
//! whether a call is well-formed enough to have one. Both checks exist because a template
//! error that renders anyway is worse than one that stops: a typo'd keyword silently
//! dropped ships a subtly wrong page, and a missing argument silently niled ships a blank
//! where content belongs.

use super::template_filter_arg::literal_of;
use super::template_macro::Macro;
use super::template_macro_arg::Arg;
use super::template_macro_report::signature;
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
pub(super) fn reject_unknown(
    def: &Macro<'_>,
    path: &str,
    args: &[Arg<'_>],
) -> Result<(), String> {
    for arg in args {
        if !def.params.iter().any(|param| param.name == arg.name) {
            return Err(format!(
                "template: macro `{path}` has no parameter `{}`; it takes {}",
                arg.name,
                signature(def)
            ));
        }
    }
    Ok(())
}

/// The declared default for an unsupplied parameter.
///
/// # Arguments
///
/// * `def` — The macro being called, for the signature in the message.
/// * `path` — Call path as written.
/// * `name` — Parameter that was not supplied.
/// * `default` — Its raw default literal, or `None` when it is required.
///
/// # Returns
///
/// The default converted by [`literal_of`], so `size="sm"` yields a str and `n=0` an int.
///
/// # Errors
///
/// Returns an error naming the required parameter when there is no default.
pub(super) fn default_of(
    def: &Macro<'_>,
    path: &str,
    name: &str,
    default: Option<&str>,
) -> Result<Value, String> {
    match default {
        Some(literal) => Ok(literal_of(literal)),
        None => Err(format!(
            "template: macro `{path}` needs argument `{name}`; it takes {}",
            signature(def)
        )),
    }
}
