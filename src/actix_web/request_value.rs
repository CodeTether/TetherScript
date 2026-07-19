//! Construction of tetherscript request values.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

pub(super) fn from_parts(
    method: String,
    path: String,
    query: String,
    headers: HashMap<String, String>,
    params: HashMap<String, String>,
    body: Vec<u8>,
) -> Value {
    let request = HashMap::from([
        ("method".into(), string(method)),
        ("path".into(), string(path)),
        ("query".into(), string(query)),
        ("headers".into(), string_map(headers)),
        ("params".into(), string_map(params)),
        ("body".into(), body_value(body)),
    ]);
    Value::Map(Rc::new(RefCell::new(request)))
}

fn string(value: String) -> Value {
    Value::Str(Rc::new(value))
}

fn string_map(values: HashMap<String, String>) -> Value {
    let values = values
        .into_iter()
        .map(|(key, value)| (key, string(value)))
        .collect();
    Value::Map(Rc::new(RefCell::new(values)))
}

fn body_value(value: Vec<u8>) -> Value {
    String::from_utf8(value)
        .map(string)
        .unwrap_or_else(|error| Value::Bytes(Rc::new(RefCell::new(error.into_bytes()))))
}
