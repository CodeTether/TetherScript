//! Status-map readers for the `chan_*` built-in tests.

use crate::value::Value;

/// Read a field from a status map.
pub(super) fn field(map: &Value, key: &str) -> Option<Value> {
    let Value::Map(fields) = map else {
        panic!("expected a status map, got {}", map.type_name());
    };
    fields.borrow().get(key).cloned()
}

/// Read the `status` field of a status map as a string.
pub(super) fn status(map: &Value) -> String {
    match field(map, "status") {
        Some(Value::Str(status)) => status.as_str().to_string(),
        other => panic!("expected a status str, got {other:?}"),
    }
}

/// Assert that a value is the given `str`.
pub(super) fn assert_text(value: &Value, expected: &str) {
    match value {
        Value::Str(actual) => assert_eq!(actual.as_str(), expected),
        other => panic!("expected a str, got {}", other.type_name()),
    }
}
