//! Validation that makes `Set-Cookie` header injection impossible.
//!
//! This is the security boundary of the cookie built-ins. A `;` in a name or
//! value would end the cookie and let the caller append attributes such as
//! `HttpOnly` or a second `Path`; a CR or LF would end the header line entirely
//! and allow injecting arbitrary further headers or a response body.
//!
//! Rejecting is the only safe answer. Silently stripping or percent-encoding the
//! offending byte would change the value the caller asked to store, so a rejected
//! value is reported instead of quietly rewritten.

use crate::value::Value;

/// Coerce a script argument to a string, naming the parameter on mismatch.
///
/// `system::string_arg` is private to its module, so cookie argument coercion is
/// duplicated here rather than widening another module's visibility.
///
/// # Errors
///
/// Returns an error naming `label` and the actual type when `value` is not a str.
pub(super) fn string_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}

/// Reject any byte that could break out of a cookie or a header line.
///
/// # Arguments
///
/// * `label` — What is being checked, used to name the failure.
/// * `text` — The candidate cookie name, value, or attribute.
///
/// # Errors
///
/// Returns an error naming `label` and the offending character when `text`
/// contains a control character (including CR, LF, and NUL) or a `;`.
///
/// `,` is deliberately allowed: `Expires` carries the RFC 7231 date form
/// `Wed, 21 Oct 2015 07:28:00 GMT`, and a comma cannot end a cookie or a header.
pub(super) fn reject_injection(label: &str, text: &str) -> Result<(), String> {
    for ch in text.chars() {
        let bad = match ch {
            ';' => "`;`",
            '\r' => "a carriage return",
            '\n' => "a newline",
            '\0' => "a NUL byte",
            // Covers the remaining C0 range and DEL.
            c if c.is_control() => "a control character",
            _ => continue,
        };
        return Err(format!(
            "cookie {label} must not contain {bad}: header injection rejected"
        ));
    }
    Ok(())
}

/// Reject a cookie name that is empty or carries separator characters.
///
/// # Errors
///
/// Returns an error when the name is empty, fails [`reject_injection`], or
/// contains `=`, a space, or a double quote, any of which would be reparsed as
/// structure rather than as part of the name.
pub(super) fn check_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("cookie name must not be empty".into());
    }
    reject_injection("name", name)?;
    for ch in ['=', ' ', '"'] {
        if name.contains(ch) {
            return Err(format!("cookie name must not contain `{ch}`"));
        }
    }
    Ok(())
}
