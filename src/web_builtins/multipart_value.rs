//! Conversion of parsed parts into script-visible values, and field lookup.
//!
//! Each part becomes a map with `name`, `filename`, `content_type`, and `body`.
//! Absent `filename` and `content_type` are `nil` rather than empty strings, so a
//! script can distinguish a file part from a plain text field, and an empty
//! filename from a missing one.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::multipart_split::Part;
use crate::value::Value;

/// Render decoded parts as a list of maps.
pub(super) fn to_value(parts: Vec<Part>) -> Value {
    let items: Vec<Value> = parts.into_iter().map(part_map).collect();
    Value::List(Rc::new(RefCell::new(items)))
}

fn part_map(part: Part) -> Value {
    let mut fields = HashMap::new();
    fields.insert("name".to_string(), optional(part.headers.name));
    fields.insert("filename".to_string(), optional(part.headers.filename));
    fields.insert(
        "content_type".to_string(),
        optional(part.headers.content_type),
    );
    fields.insert("body".to_string(), Value::Str(Rc::new(part.body)));
    Value::Map(Rc::new(RefCell::new(fields)))
}

/// A missing header is `nil`; an empty one stays an empty string.
fn optional(value: Option<String>) -> Value {
    match value {
        Some(text) => Value::Str(Rc::new(text)),
        None => Value::Nil,
    }
}

/// Return the body of the first part whose `name` matches.
///
/// # Arguments
///
/// * `parts` — The list returned by `multipart_parse`.
/// * `name` — Field name to find.
///
/// # Returns
///
/// The matching part's body.
///
/// # Errors
///
/// Returns an error naming the field when no part carries it, and when `parts` is
/// not a list of maps, so a mis-plumbed caller sees the real problem.
pub(super) fn field(parts: &Value, name: &str) -> Result<Value, String> {
    let Value::List(items) = parts else {
        return Err(format!(
            "multipart_field: parts must be a list, got {}",
            parts.type_name()
        ));
    };
    for item in items.borrow().iter() {
        let Value::Map(fields) = item else { continue };
        let fields = fields.borrow();
        if let Some(Value::Str(found)) = fields.get("name") {
            if found.as_str() == name {
                return Ok(fields.get("body").cloned().unwrap_or(Value::Nil));
            }
        }
    }
    Err(format!("multipart_field: no part named `{name}`"))
}
