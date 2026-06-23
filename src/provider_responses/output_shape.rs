//! Chat-completions-shaped provider response construction.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

pub(super) fn chat(content: String, tool_calls: Vec<Value>) -> Value {
    let mut message = HashMap::new();
    message.insert("role".into(), string("assistant"));
    message.insert("content".into(), string(content));
    if !tool_calls.is_empty() {
        message.insert("tool_calls".into(), list(tool_calls));
    }
    let mut choice = HashMap::new();
    choice.insert("message".into(), map(message));
    let mut root = HashMap::new();
    root.insert("choices".into(), list(vec![map(choice)]));
    map(root)
}

pub(super) fn map(values: HashMap<String, Value>) -> Value {
    Value::Map(Rc::new(RefCell::new(values)))
}

pub(super) fn list(values: Vec<Value>) -> Value {
    Value::List(Rc::new(RefCell::new(values)))
}

pub(super) fn string(text: impl Into<String>) -> Value {
    Value::Str(Rc::new(text.into()))
}
