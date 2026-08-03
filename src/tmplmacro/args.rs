//! Macro call argument parsing.
//!
//! Arguments are **keyword-only**, as Tera's are: `row(cfg=item, wide=true)`.
//!
//! # Positional arguments
//!
//! Positional arguments are deliberately *not* supported. This was checked against the
//! reference application rather than assumed: every namespaced macro call site in
//! `.tera` views is keyword form, and a scan for the positional shape
//! `ns::name(<no '='>)` returns nothing. Accepting position would make `badge("new")`
//! bind silently to whichever parameter happens to be declared first, so reordering a
//! header would silently rewire every call site. An argument with no `=` is therefore
//! reported as an error, which also catches the plain typo case.
//!
//! Splitting is quote- and paren-aware via [`crate::tmplmacro::split`]: a literal such as
//! `sep=", "` or `label="Book (today)"` legally contains the separator.

use crate::tmplmacro::split::split_outside_quotes;

/// One supplied argument: its keyword and the raw expression text for its value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arg<'a> {
    /// Keyword naming the parameter to bind.
    pub name: &'a str,
    /// Raw value expression: a literal, or a possibly-dotted caller-context key.
    pub expression: &'a str,
}

/// Parse the comma-separated interior of a call's argument list.
///
/// # Arguments
///
/// * `arguments` — Trimmed text between the call's parentheses; may be empty.
/// * `path` — Call path, used only to name the offender in an error message.
///
/// # Returns
///
/// The arguments in source order; an empty list for `name()`.
///
/// # Errors
///
/// Returns an error for an argument with no `=` — that is either a positional argument or
/// a typo, and both must be reported — or for an empty keyword.
///
/// # Examples
///
/// ```
/// use tetherscript::tmplmacro::args::parse_args;
///
/// let args = parse_args(r#"kind="a, b", size=s"#, "ui::badge").unwrap();
/// assert_eq!(args.len(), 2);
/// assert_eq!(args[0].expression, r#""a, b""#);
/// assert!(parse_args(r#""new""#, "ui::badge").is_err());
/// ```
pub fn parse_args<'a>(arguments: &'a str, path: &str) -> Result<Vec<Arg<'a>>, String> {
    if arguments.is_empty() {
        return Ok(Vec::new());
    }
    split_outside_quotes(arguments, ',')
        .into_iter()
        .map(|part| one_arg(part.trim(), path))
        .collect()
}

/// Parse a single `name=<expression>` argument.
fn one_arg<'a>(text: &'a str, path: &str) -> Result<Arg<'a>, String> {
    let (name, expression) = text.split_once('=').ok_or_else(|| {
        format!(
            "template: argument `{text}` to macro `{path}` must be `name=value`; \
             macro arguments are keyword-only"
        )
    })?;
    let name = name.trim();
    if name.is_empty() {
        return Err(format!(
            "template: empty argument name in call to macro `{path}`"
        ));
    }
    Ok(Arg {
        name,
        expression: expression.trim(),
    })
}
