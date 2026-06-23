//! Conversion from key events to tetherscript values.

use crate::value::Value;

use super::{key_event::KeyEvent, val};

/// Convert a parsed key event into a script-visible map.
pub(super) fn to_value(event: KeyEvent) -> Value {
    val::map_value([
        ("type".into(), val::strv("key")),
        ("key".into(), val::strv(event.key)),
        ("text".into(), text_value(event.text)),
        ("ctrl".into(), Value::Bool(event.ctrl)),
        ("alt".into(), Value::Bool(event.alt)),
        ("shift".into(), Value::Bool(event.shift)),
    ])
}

fn text_value(text: Option<String>) -> Value {
    match text {
        Some(text) => val::strv(text),
        None => Value::Nil,
    }
}
