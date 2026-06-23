//! JSON-RPC error message constructor.

use crate::value::Value;

use super::jsonrpc;
use super::val;

pub(super) fn error(args: &[Value]) -> Result<Value, String> {
    if !(3..=4).contains(&args.len()) {
        return Err("jsonrpc_error expects id, code, message[, data]".into());
    }
    let Value::Int(_) = args[1] else {
        return Err(format!(
            "jsonrpc_error: code must be int, got {}",
            args[1].type_name()
        ));
    };
    let mut fields = vec![
        ("code".into(), args[1].clone()),
        (
            "message".into(),
            jsonrpc::string(&args[2], "jsonrpc_error: message")?,
        ),
    ];
    if args.len() == 4 {
        fields.push(("data".into(), args[3].clone()));
    }
    Ok(jsonrpc::message(vec![
        ("id", args[0].clone()),
        ("error", val::map_value(fields)),
    ]))
}
