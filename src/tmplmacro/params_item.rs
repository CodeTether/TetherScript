//! Parameter-list item parsing: one `name` or `name="default"` entry.
//!
//! Separated from header parsing so the [`Param`] shape and the list grammar live in one
//! place. Splitting is quote-aware because a default literal may legally contain a comma,
//! as in `sep=", "`.

use crate::tmplmacro::split::split_outside_quotes;

/// One declared parameter: its name and optional raw default literal.
///
/// The default is stored as source text, not a value, so conversion is deferred to
/// [`crate::tmplmacro::literal::literal_of`] at bind time.
///
/// # Examples
///
/// ```
/// use tetherscript::tmplmacro::params_item::Param;
///
/// let required = Param { name: "kind".into(), default: None };
/// assert!(required.default.is_none());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    /// Parameter name as written.
    pub name: String,
    /// Raw default literal text, or `None` when the parameter is required.
    pub default: Option<String>,
}

/// Parse the comma-separated interior of a parameter list.
///
/// # Arguments
///
/// * `text` — Trimmed text between the header's parentheses; may be empty.
///
/// # Returns
///
/// The parameters in source order; an empty list when `text` is empty.
///
/// # Errors
///
/// Returns an error when a parameter name is empty, e.g. `macro n(a,,b)`.
///
/// # Examples
///
/// ```
/// use tetherscript::tmplmacro::params_item::parse_params;
///
/// let params = parse_params(r#"kind, size="sm""#).unwrap();
/// assert_eq!(params[0].name, "kind");
/// assert!(parse_params("a,,b").is_err());
/// ```
pub fn parse_params(text: &str) -> Result<Vec<Param>, String> {
    if text.is_empty() {
        return Ok(Vec::new());
    }
    split_outside_quotes(text, ',')
        .into_iter()
        .map(|part| one_param(part.trim()))
        .collect()
}

/// Parse one `name` or `name=<literal>` parameter.
fn one_param(text: &str) -> Result<Param, String> {
    let (name, default) = match text.split_once('=') {
        Some((name, default)) => (name.trim(), Some(default.trim().to_string())),
        None => (text, None),
    };
    if name.is_empty() {
        return Err("template: empty parameter name in macro header".to_string());
    }
    Ok(Param {
        name: name.to_string(),
        default,
    })
}
