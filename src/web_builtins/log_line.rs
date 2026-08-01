//! Building a structured log line.
//!
//! The line is assembled as a [`Value::Map`] and handed to the in-tree JSON
//! encoder rather than formatted by hand. That is deliberate: a message
//! containing a quote, a newline, or a backslash would break a hand-built line
//! and produce output no log collector can parse.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::log_level;
use crate::json;
use crate::system::time_now_ms;
use crate::value::Value;

/// Keys the caller may not overwrite.
///
/// A caller field named `level` must not be able to relabel the severity of its
/// own line, which would let a script hide an error by shadowing the key.
pub(super) const RESERVED: [&str; 3] = ["level", "msg", "ts"];

/// Render one log line as compact JSON.
///
/// # Arguments
///
/// * `level` — Severity name; validated and lowercased.
/// * `message` — Human-readable message, stored under `msg`.
/// * `fields` — Optional caller map merged in around the reserved keys.
///
/// # Returns
///
/// The encoded JSON line, without a trailing newline.
///
/// # Errors
///
/// Returns an error when `level` is unknown, when `fields` is neither a map nor
/// nil, or when the assembled line cannot be JSON-encoded.
pub(super) fn render(level: &str, message: &str, fields: &Value) -> Result<String, String> {
    let mut line: HashMap<String, Value> = HashMap::new();

    // Caller fields go in first, so the reserved keys below always win.
    match fields {
        Value::Nil => {}
        Value::Map(map) => {
            for (key, value) in map.borrow().iter() {
                if RESERVED.contains(&key.as_str()) {
                    continue;
                }
                line.insert(key.clone(), value.clone());
            }
        }
        other => {
            return Err(format!(
                "log: fields must be a map or nil, got {}",
                other.type_name()
            ));
        }
    }

    line.insert(
        "level".into(),
        Value::Str(Rc::new(log_level::canonical(level)?.to_string())),
    );
    line.insert("msg".into(), Value::Str(Rc::new(message.to_string())));
    line.insert("ts".into(), time_now_ms());

    json::encode_to_string(&Value::Map(Rc::new(RefCell::new(line))))
        .map_err(|error| format!("log: cannot encode line: {error}"))
}
