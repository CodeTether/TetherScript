//! `Cookie` header parsing.
//!
//! Splits a request `Cookie` header into name/value pairs. Parsing is deliberately
//! lenient — a malformed pair is skipped rather than failing the whole request,
//! because a browser may send cookies this server never set.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

/// Parse a `Cookie` header into a map of names to values.
///
/// Pairs are separated by `;`, and surrounding whitespace is trimmed. Only the
/// first `=` splits a pair, so a value may itself contain `=` (base64 padding and
/// signed session cookies both rely on this). A double-quoted value is unwrapped,
/// per the quoted-string form in RFC 6265.
///
/// A pair with no `=`, or with an empty name, is skipped. When a name repeats,
/// the last occurrence wins.
pub(super) fn parse(header: &str) -> Value {
    let mut jar: HashMap<String, Value> = HashMap::new();
    for pair in header.split(';') {
        let Some((name, value)) = pair.split_once('=') else {
            continue;
        };
        let name = name.trim();
        if name.is_empty() {
            continue;
        }
        let value = unquote(value.trim());
        jar.insert(name.to_string(), Value::Str(Rc::new(value)));
    }
    Value::Map(Rc::new(RefCell::new(jar)))
}

/// Strip one layer of surrounding double quotes, if both are present.
fn unquote(value: &str) -> String {
    let bytes = value.as_bytes();
    if bytes.len() >= 2 && bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"' {
        return value[1..value.len() - 1].to_string();
    }
    value.to_string()
}
