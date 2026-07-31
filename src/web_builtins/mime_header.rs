//! `Content-Type` header parsing and textual-type classification.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

/// Parse a `Content-Type` header into its type and parameters.
///
/// # Arguments
///
/// * `header` — Header value such as `text/html; charset=utf-8`.
///
/// # Returns
///
/// A map whose `type` key holds the lowercase media type, plus one lowercase key
/// per parameter. Quoted parameter values are unquoted.
///
/// # Errors
///
/// Returns an error when the media type is empty, so a caller cannot mistake a
/// blank header for a successfully parsed one.
pub(super) fn parse(header: &str) -> Result<Value, String> {
    // Split on `;` only outside quotes: a multipart boundary may legally contain
    // one, and splitting first would truncate the value mid-token.
    let mut parts = split_unquoted(header).into_iter();
    let media = parts.next().unwrap_or_default().trim().to_ascii_lowercase();
    if media.is_empty() {
        return Err("mime_parse: header has no media type".to_string());
    }

    let mut fields = HashMap::new();
    fields.insert("type".to_string(), Value::Str(Rc::new(media)));
    for part in parts {
        let Some((name, value)) = part.split_once('=') else {
            continue;
        };
        let name = name.trim().to_ascii_lowercase();
        if name.is_empty() {
            continue;
        }
        fields.insert(name, Value::Str(Rc::new(unquote(value.trim()))));
    }
    Ok(Value::Map(Rc::new(RefCell::new(fields))))
}

/// Split on `;` while treating a double-quoted run as opaque.
fn split_unquoted(header: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    for ch in header.chars() {
        match ch {
            '"' => {
                quoted = !quoted;
                current.push(ch);
            }
            ';' if !quoted => parts.push(std::mem::take(&mut current)),
            _ => current.push(ch),
        }
    }
    parts.push(current);
    parts
}

/// Strip surrounding double quotes and unescape `\"` inside a quoted string.
fn unquote(value: &str) -> String {
    let inner = value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(value);
    inner.replace("\\\"", "\"")
}
