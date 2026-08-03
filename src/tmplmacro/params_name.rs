//! Identifier validation for macro names and call paths.
//!
//! One concern: deciding what counts as a name. Rejecting a non-identifier at collection
//! time means a header typo surfaces even for a macro that is never called, rather than
//! lying in wait until the one page that uses it is requested.

/// Whether `name` is a legal macro or namespace identifier.
///
/// # Arguments
///
/// * `name` — Candidate identifier.
///
/// # Returns
///
/// True for a non-empty run of ASCII alphanumerics and `_` that does not start with a
/// digit.
///
/// # Examples
///
/// ```
/// use tetherscript::tmplmacro::params_name::is_identifier;
///
/// assert!(is_identifier("service_calendar"));
/// assert!(!is_identifier("2col"));
/// assert!(!is_identifier(""));
/// ```
pub fn is_identifier(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !name.starts_with(|c: char| c.is_ascii_digit())
}

/// Reject a macro name that is not an identifier.
///
/// # Arguments
///
/// * `name` — Candidate macro name.
/// * `body` — Full header text, quoted in the error so the site is findable.
///
/// # Returns
///
/// `Ok(())` when `name` is an identifier.
///
/// # Errors
///
/// Returns an error naming both the offending name and its header.
///
/// # Examples
///
/// ```
/// use tetherscript::tmplmacro::params_name::reject_bad_name;
///
/// assert!(reject_bad_name("badge", "macro badge()").is_ok());
/// assert!(reject_bad_name("", "macro ()").is_err());
/// ```
pub fn reject_bad_name(name: &str, body: &str) -> Result<(), String> {
    if is_identifier(name) {
        return Ok(());
    }
    Err(format!(
        "template: macro name `{name}` in `{body}` is not an identifier"
    ))
}
