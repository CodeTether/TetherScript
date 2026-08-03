//! Rendering a macro's signature for error messages.
//!
//! Every rejection quotes the signature, because "unknown argument `wide`" is far less
//! actionable than the same message followed by `badge(kind, size="sm")`. One concern:
//! formatting, no policy.

use crate::tmplmacro::macros::MacroDef;

/// Format a macro's declared signature as it would be written.
///
/// # Arguments
///
/// * `def` — The macro definition to describe.
///
/// # Returns
///
/// A string such as `badge(kind, size="sm")`, or `row()` for a parameterless macro.
///
/// # Examples
///
/// ```
/// use tetherscript::tmplmacro::macros::collect;
/// use tetherscript::tmplmacro::report::signature;
///
/// let set = collect(r#"{% macro badge(kind, size="sm") %}{% endmacro %}"#).unwrap();
/// assert_eq!(signature(&set["badge"]), r#"badge(kind, size="sm")"#);
/// ```
pub fn signature(def: &MacroDef) -> String {
    let params: Vec<String> = def
        .params
        .iter()
        .map(|param| match &param.default {
            Some(default) => format!("{}={default}", param.name),
            None => param.name.clone(),
        })
        .collect();
    format!("{}({})", def.name, params.join(", "))
}
