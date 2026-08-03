//! Call-site classification and parsing for `namespace::name(args)` and bare `name(args)`.
//!
//! In the reference application a macro is invoked through a **hole**, not a tag:
//! `{{ booking::service_calendar(cfg=x) }}`. So the engine reaches this from expression
//! emission, and [`is_call`] exists to classify a hole body before committing to the macro
//! path. A filter pipeline such as `x | default(value="")` must not be mistaken for a call,
//! so the head before `(` is required to be a bare identifier path — a pipeline's head
//! contains a space and a `|`, and so is rejected.
//!
//! Path splitting lives in [`crate::tmplmacro::call_path`]; argument syntax in
//! [`crate::tmplmacro::args`].

use crate::tmplmacro::args::{parse_args, Arg};
use crate::tmplmacro::call_path::{split_call, split_path};
use crate::tmplmacro::params_name::is_identifier;

/// One parsed call site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call<'a> {
    /// Namespace, or `None` for `self::`, `_self::`, and the bare form.
    pub namespace: Option<&'a str>,
    /// Macro name.
    pub name: &'a str,
    /// Path as written, retained verbatim for error messages.
    pub path: &'a str,
    /// Supplied keyword arguments, in source order.
    pub args: Vec<Arg<'a>>,
}

/// Whether a hole body is a macro call rather than a value expression.
///
/// # Arguments
///
/// * `body` — Trimmed text from inside `{{ }}`.
///
/// # Returns
///
/// True when the body is `path(...)` and `path` is an identifier path of alphanumerics,
/// `_`, and `::`.
///
/// # Examples
///
/// ```
/// use tetherscript::tmplmacro::call::is_call;
///
/// assert!(is_call(r#"ui::badge(kind="new")"#));
/// assert!(is_call("row()"));
/// assert!(!is_call(r#"cfg.html | default(value="")"#));
/// assert!(!is_call("cfg.title"));
/// ```
pub fn is_call(body: &str) -> bool {
    let body = body.trim();
    let Some(open) = body.find('(') else {
        return false;
    };
    let head = body[..open].trim();
    body.ends_with(')') && !head.is_empty() && head.split("::").all(is_identifier)
}

/// Parse a call body into its path, namespace, name, and arguments.
///
/// # Arguments
///
/// * `body` — Trimmed hole body, such as `ui::badge(kind="new")`.
///
/// # Returns
///
/// The parsed [`Call`], borrowing from `body`.
///
/// # Errors
///
/// Returns an error when the parentheses are missing or unbalanced, the path is not an
/// identifier path, or an argument is not `name=value`.
///
/// # Examples
///
/// ```
/// use tetherscript::tmplmacro::call::parse_call;
///
/// let call = parse_call("booking::service_calendar(quote_id='q-1')").unwrap();
/// assert_eq!(call.namespace, Some("booking"));
/// assert_eq!(call.name, "service_calendar");
/// assert_eq!(call.args[0].name, "quote_id");
///
/// let own = parse_call("self::row(cfg=item)").unwrap();
/// assert_eq!(own.namespace, None);
/// assert_eq!(own.name, "row");
/// ```
pub fn parse_call(body: &str) -> Result<Call<'_>, String> {
    let (path, inner) = split_call(body)?;
    let (namespace, name) = split_path(path);
    if !is_identifier(name) || namespace.is_some_and(|ns| !is_identifier(ns)) {
        return Err(format!("template: `{path}` is not a macro call path"));
    }
    Ok(Call {
        namespace,
        name,
        path,
        args: parse_args(inner, path)?,
    })
}
