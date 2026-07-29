//! JSON-RPC request encoding for the agent child process.

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::value::Value;

pub(crate) fn encode(id: i64, method: &str, prompt: Option<&str>) -> Result<String, String> {
    let mut root = HashMap::new();
    root.insert("jsonrpc".into(), string("2.0"));
    root.insert("id".into(), Value::Int(id));
    root.insert("method".into(), string(method));
    if let Some(prompt) = prompt {
        let mut params = HashMap::new();
        params.insert("prompt".into(), string(prompt));
        root.insert("params".into(), Value::Map(Rc::new(RefCell::new(params))));
    }
    crate::json::encode_to_string(&Value::Map(Rc::new(RefCell::new(root))))
}

fn string(value: &str) -> Value {
    Value::Str(Rc::new(value.into()))
}
