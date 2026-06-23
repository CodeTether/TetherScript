//! Responses function-call conversion.

use std::collections::HashMap;

use crate::value::Value;

use super::{output_fields as fields, output_shape as shape};

pub(super) fn tool_call(item: &Value) -> Result<Value, String> {
    let id = fields::string(item, "call_id")
        .or_else(|| fields::string(item, "id"))
        .ok_or("provider.responses: function_call missing call_id/id")?;
    let name =
        fields::string(item, "name").ok_or("provider.responses: function_call missing name")?;
    let arguments = match fields::field(item, "arguments") {
        Some(Value::Str(text)) => text.to_string(),
        Some(value) => crate::json::encode_to_string(&value)
            .map_err(|error| format!("provider.responses: encode arguments: {error}"))?,
        None => return Err("provider.responses: function_call missing arguments".into()),
    };
    let mut function = HashMap::new();
    function.insert("name".into(), shape::string(name));
    function.insert("arguments".into(), shape::string(arguments));
    let mut call = HashMap::new();
    call.insert("id".into(), shape::string(id.clone()));
    call.insert("call_id".into(), shape::string(id));
    call.insert("type".into(), shape::string("function"));
    call.insert("function".into(), shape::map(function));
    Ok(shape::map(call))
}
