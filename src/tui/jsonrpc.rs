//! JSON-RPC 2.0 message constructors.

use crate::value::Value;

use super::val;

pub(super) fn request(args: &[Value]) -> Result<Value, String> {
    arity(args, 3, "jsonrpc_request")?;
    Ok(message(vec![
        ("id", args[0].clone()),
        ("method", string(&args[1], "jsonrpc_request: method")?),
        ("params", args[2].clone()),
    ]))
}

pub(super) fn response(args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "jsonrpc_response")?;
    Ok(message(vec![
        ("id", args[0].clone()),
        ("result", args[1].clone()),
    ]))
}

pub(super) fn notify(args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "jsonrpc_notify")?;
    Ok(message(vec![
        ("method", string(&args[0], "jsonrpc_notify: method")?),
        ("params", args[1].clone()),
    ]))
}

pub(super) fn message(items: Vec<(&str, Value)>) -> Value {
    let mut fields = vec![("jsonrpc".into(), val::strv("2.0"))];
    fields.extend(items.into_iter().map(|(key, value)| (key.into(), value)));
    val::map_value(fields)
}

pub(super) fn arity(args: &[Value], len: usize, name: &str) -> Result<(), String> {
    if args.len() == len {
        Ok(())
    } else {
        Err(format!("{name} expects {len} args"))
    }
}

pub(super) fn string(value: &Value, label: &str) -> Result<Value, String> {
    match value {
        Value::Str(_) => Ok(value.clone()),
        other => Err(format!("{label} must be str, got {}", other.type_name())),
    }
}
