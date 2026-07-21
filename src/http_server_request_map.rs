//! Construction of the language-level HTTP request map.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

pub(super) fn build(
    method: String,
    target: String,
    headers: HashMap<String, Value>,
    body: String,
) -> Value {
    let (path, query) = match target.split_once('?') {
        Some((path, query)) => (path.to_string(), query.to_string()),
        None => (target, String::new()),
    };
    let mut request = HashMap::new();
    request.insert("method".into(), Value::Str(Rc::new(method)));
    request.insert("path".into(), Value::Str(Rc::new(path)));
    request.insert("query".into(), Value::Str(Rc::new(query)));
    request.insert("headers".into(), Value::Map(Rc::new(RefCell::new(headers))));
    request.insert("body".into(), Value::Str(Rc::new(body)));
    Value::Map(Rc::new(RefCell::new(request)))
}
