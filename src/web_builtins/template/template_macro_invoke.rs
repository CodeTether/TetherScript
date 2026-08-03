//! Macro lookup and body rendering.
//!
//! The step between "this is a call" and "here is its output": find the definition, bind a
//! scope, render. Escaping is inherited from the enclosing render rather than reset, so a
//! `{{ }}` inside a macro body is escaped exactly as it would be outside one, and `| safe`
//! inside the body still opts out per hole. Resetting escaping here would have opened an
//! XSS hole in every macro-rendered fragment.

use super::template_block::{render_with, Render};
use super::template_macro::Macros;
use super::template_macro_arg::parse;
use super::template_macro_body::assemble;
use super::template_macro_scope::bind;
use crate::value::Value;

/// Look up and render one macro.
///
/// # Arguments
///
/// * `macros` — Macros declared by the defining template.
/// * `namespace` — Namespace as written, or `self` for the current template. Used only in
///   error messages.
/// * `name` — Macro name to call.
/// * `arguments` — Raw text between the call's parentheses.
/// * `context` — Caller context, consulted only to evaluate argument expressions.
/// * `state` — Enclosing render state; the body renders one level deeper.
///
/// # Returns
///
/// The rendered macro output.
///
/// # Errors
///
/// Returns an error naming the namespace and the macro when no such macro is declared,
/// and propagates argument-binding and body-rendering failures unchanged.
pub(super) fn invoke(
    macros: &Macros<'_>,
    namespace: &str,
    name: &str,
    arguments: &str,
    context: &Value,
    state: &Render<'_>,
) -> Result<String, String> {
    let path = format!("{namespace}::{name}");
    let def = macros.get(name).ok_or_else(|| {
        format!(
            "template: unknown macro `{path}`; `{namespace}` defines {}",
            declared(macros)
        )
    })?;
    let args = parse(arguments, &path)?;
    let scope = bind(def, &path, &args, context)?;
    let pieces = assemble(macros, def);
    // `deeper` bounds recursion on the same counter `include` uses.
    let nested = state.nested(state.escaping, true);
    render_with(&pieces, &scope, &nested)
}

/// Name the macros a template declares, for an unknown-macro error.
fn declared(macros: &Macros<'_>) -> String {
    if macros.is_empty() {
        return "no macros".to_string();
    }
    let mut names: Vec<&str> = macros.keys().map(String::as_str).collect();
    // Sorted so the message is stable across runs; HashMap order is not.
    names.sort_unstable();
    names.join(", ")
}
