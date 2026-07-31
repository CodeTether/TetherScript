//! `Set-Cookie` header construction.
//!
//! Attribute order follows the reference Actix behaviour in the reference application:
//! `Path`, `Domain`, `Max-Age`, `Expires`, then the flags. Every attribute value
//! passes through [`super::cookie_guard::reject_injection`] before it is appended,
//! so an attribute can never introduce a second cookie or header.

use std::collections::HashMap;

use super::cookie_alias;
use super::cookie_guard::reject_injection;
use super::cookie_options as options;
use crate::value::Value;

/// Build a `Set-Cookie` header value.
///
/// # Errors
///
/// Returns an error when the name, value, or any attribute fails validation, or
/// when `SameSite` is not `Strict`, `Lax`, or `None`.
pub(super) fn serialize(
    name: &str,
    value: &str,
    opts: &HashMap<String, Value>,
) -> Result<String, String> {
    let mut header = format!("{name}={value}");

    for (key, attribute) in [("Path", "path"), ("Domain", "domain")] {
        if let Some(text) = options::string(opts, attribute)? {
            reject_injection(attribute, &text)?;
            header.push_str(&format!("; {key}={text}"));
        }
    }

    if let Some(max_age) = options::integer(opts, "max_age")? {
        header.push_str(&format!("; Max-Age={max_age}"));
    }
    if let Some(expires) = options::string(opts, "expires")? {
        reject_injection("expires", &expires)?;
        header.push_str(&format!("; Expires={expires}"));
    }
    if let Some(same_site) = options::string(opts, "same_site")? {
        header.push_str(&format!(
            "; SameSite={}",
            cookie_alias::same_site(&same_site)?
        ));
    }
    if options::flag(opts, "http_only")? {
        header.push_str("; HttpOnly");
    }
    if options::flag(opts, "secure")? {
        header.push_str("; Secure");
    }
    Ok(header)
}
