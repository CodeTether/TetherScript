//! Argument coercion and tetherscript value construction.
//!
//! Kept separate from the matching logic so the pure Rust matcher stays free of
//! `Value` plumbing and can be reasoned about on its own.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

use super::route_decode::decode;
use super::route_match::{match_path, params};
use super::route_segments::split;

/// Extract a str argument, naming the builtin and parameter on failure.
fn str_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok((**text).clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}

/// `route_match(pattern, path)` -> map of captures, or nil when it does not match.
pub(super) fn match_builtin(args: &[Value]) -> Result<Value, String> {
    let pattern = str_arg(&args[0], "route_match: pattern")?;
    let path = str_arg(&args[1], "route_match: path")?;
    let Some(captures) = match_path(&pattern, &path)? else {
        // A miss is an ordinary outcome, so it is nil rather than an Err.
        return Ok(Value::Nil);
    };
    let mut map = HashMap::new();
    for (name, value) in captures {
        map.insert(name, Value::Str(Rc::new(value)));
    }
    Ok(Value::Map(Rc::new(RefCell::new(map))))
}

/// `route_params(pattern)` -> list of declared parameter names, in order.
pub(super) fn params_builtin(args: &[Value]) -> Result<Value, String> {
    let pattern = str_arg(&args[0], "route_params: pattern")?;
    let names = params(&pattern)?
        .into_iter()
        .map(|name| Value::Str(Rc::new(name)))
        .collect();
    Ok(Value::List(Rc::new(RefCell::new(names))))
}

/// `path_segments(path)` -> list of non-empty, percent-decoded segments.
pub(super) fn segments_builtin(args: &[Value]) -> Result<Value, String> {
    let path = str_arg(&args[0], "path_segments: path")?;
    let mut out = Vec::new();
    for segment in split(&path) {
        out.push(Value::Str(Rc::new(decode(segment)?)));
    }
    Ok(Value::List(Rc::new(RefCell::new(out))))
}
