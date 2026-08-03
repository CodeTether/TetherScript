//! JSON shaping helpers for LSP replies.
//!
//! `src/lsp.rs` keeps its own private copies of these constructors and
//! accessors. They are private, so this module re-states them rather than
//! reaching into the server, keeping the two files independently testable
//! while producing byte-identical JSON: objects are [`Value::Map`], arrays are
//! [`Value::List`], and strings are [`Value::Str`], exactly as the in-tree
//! JSON encoder used by the server expects.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::jsonval::{field, obj, str_value, ValueText};
//!
//! let value = obj(vec![("kind", str_value("markdown"))]);
//! assert_eq!(field(&value, "kind").as_deref_str(), Some("markdown"));
//! ```

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

/// Build a JSON object from ordered `(key, value)` pairs.
///
/// # Arguments
///
/// * `fields` — Key/value pairs; later duplicates overwrite earlier ones.
///
/// # Returns
///
/// A [`Value::Map`] suitable for the in-tree JSON encoder.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::jsonval::{obj, str_value};
/// let value = obj(vec![("label", str_value("println"))]);
/// assert!(matches!(value, tetherscript::value::Value::Map(_)));
/// ```
pub fn obj(fields: Vec<(&str, Value)>) -> Value {
    let map: HashMap<String, Value> = fields
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect();
    Value::Map(Rc::new(RefCell::new(map)))
}

/// Build a JSON array.
///
/// # Arguments
///
/// * `items` — Elements of the array, in order.
///
/// # Returns
///
/// A [`Value::List`].
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::jsonval::list;
/// assert!(matches!(list(Vec::new()), tetherscript::value::Value::List(_)));
/// ```
pub fn list(items: Vec<Value>) -> Value {
    Value::List(Rc::new(RefCell::new(items)))
}

/// Build a JSON string value.
///
/// # Arguments
///
/// * `value` — Text to wrap.
///
/// # Returns
///
/// A [`Value::Str`].
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::jsonval::str_value;
/// assert!(matches!(str_value("x"), tetherscript::value::Value::Str(_)));
/// ```
pub fn str_value(value: &str) -> Value {
    Value::Str(Rc::new(value.to_string()))
}

/// Read one field of a JSON object.
///
/// # Arguments
///
/// * `value` — Value expected to be an object.
/// * `name` — Field name.
///
/// # Returns
///
/// The field, or [`Value::Nil`] when absent or when `value` is not an object.
/// Returning `Nil` rather than `Option` keeps request parsing flat: a malformed
/// request degrades to defaults instead of branching at every step.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::jsonval::{field, obj, str_value, ValueText};
/// let value = obj(vec![("uri", str_value("file:///a.tether"))]);
/// assert_eq!(field(&value, "uri").as_deref_str(), Some("file:///a.tether"));
/// assert!(matches!(field(&value, "nope"), tetherscript::value::Value::Nil));
/// ```
pub fn field(value: &Value, name: &str) -> Value {
    match value {
        Value::Map(map) => map.borrow().get(name).cloned().unwrap_or(Value::Nil),
        _ => Value::Nil,
    }
}

/// Follow a chain of object field names.
///
/// # Arguments
///
/// * `value` — Root value.
/// * `path` — Field names to traverse in order.
///
/// # Returns
///
/// The nested value, or [`Value::Nil`] if any step is missing.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::jsonval::{obj, pointer, str_value, ValueText};
/// let params = obj(vec![("textDocument", obj(vec![("uri", str_value("u"))]))]);
/// assert_eq!(pointer(&params, &["textDocument", "uri"]).as_deref_str(), Some("u"));
/// ```
pub fn pointer(value: &Value, path: &[&str]) -> Value {
    let mut current = value.clone();
    for part in path {
        current = field(&current, part);
    }
    current
}

/// Accessors used when destructuring LSP request params.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::jsonval::ValueText;
/// use tetherscript::value::Value;
/// assert_eq!(Value::Int(3).as_index(), Some(3));
/// assert_eq!(Value::Int(-1).as_index(), None);
/// ```
pub trait ValueText {
    /// Borrowed text when the value is a string, otherwise `None`.
    fn as_deref_str(&self) -> Option<&str>;
    /// Non-negative integer when the value is an integer, otherwise `None`.
    fn as_index(&self) -> Option<usize>;
}

impl ValueText for Value {
    fn as_deref_str(&self) -> Option<&str> {
        match self {
            Value::Str(text) => Some(text.as_str()),
            _ => None,
        }
    }

    fn as_index(&self) -> Option<usize> {
        match self {
            Value::Int(number) if *number >= 0 => Some(*number as usize),
            _ => None,
        }
    }
}
