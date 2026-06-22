//! Terminal size discovery without platform libraries.

use std::env;

use crate::value::Value;

use super::val;

pub(super) fn builtin(_: &[Value]) -> Result<Value, String> {
    Ok(val::map_value([
        ("cols".into(), Value::Int(read("COLUMNS", 80))),
        ("rows".into(), Value::Int(read("LINES", 24))),
    ]))
}

fn read(name: &str, default: i64) -> i64 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}
