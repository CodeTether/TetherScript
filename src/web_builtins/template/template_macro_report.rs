//! Signature rendering for macro error messages.
//!
//! Its own file because it is its own concern, and because every arity error in
//! [`super::template_macro_scope`] quotes it: naming only the offending argument tells a
//! reader what is wrong but not what to write instead.

use super::template_macro::Macro;

/// Render a macro's parameter list as human-readable text.
///
/// # Arguments
///
/// * `def` — The macro definition to describe.
///
/// # Returns
///
/// Text such as `(kind, size="sm")`, or `no arguments` when the macro declares none.
///
/// # Errors
///
/// None; this cannot fail, so it returns text rather than a `Result`.
///
/// # Examples
///
/// ```text
/// \{% macro badge(kind, size="sm") %\}  →  (kind, size="sm")
/// \{% macro spacer() %\}                →  no arguments
/// ```
pub(super) fn signature(def: &Macro<'_>) -> String {
    if def.params.is_empty() {
        return "no arguments".to_string();
    }
    let listed: Vec<String> = def
        .params
        .iter()
        .map(|param| match param.default {
            Some(literal) => format!("{}={literal}", param.name),
            None => param.name.to_string(),
        })
        .collect();
    format!("({})", listed.join(", "))
}
