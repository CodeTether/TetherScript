use crate::value::Value;

use super::{fields, string};

pub(super) fn push_calls(items: &mut Vec<String>, item: &Value) -> Result<(), String> {
    for call in fields::tool_calls(item) {
        if let Some(item) = function_call(&call)? {
            items.push(item);
        }
    }
    Ok(())
}

pub(super) fn push_output(items: &mut Vec<String>, item: &Value) -> Result<(), String> {
    let Some(id) = fields::call_id(item) else {
        return Ok(());
    };
    let Some(output) = fields::content(item) else {
        return Ok(());
    };
    items.push(format!(
        "{{\"type\":\"function_call_output\",\"call_id\":{},\"output\":{}}}",
        string(&id)?,
        string(&output)?
    ));
    Ok(())
}

fn function_call(call: &Value) -> Result<Option<String>, String> {
    let Some(id) = fields::call_id(call) else {
        return Ok(None);
    };
    let Some(name) = call_name(call) else {
        return Ok(None);
    };
    let args = call_args(call).unwrap_or_else(|| "{}".into());
    Ok(Some(format!(
        "{{\"type\":\"function_call\",\"call_id\":{},\"name\":{},\"arguments\":{}}}",
        string(&id)?,
        string(&name)?,
        string(&args)?
    )))
}

fn call_name(call: &Value) -> Option<String> {
    fields::function(call)
        .and_then(|func| fields::name(&func))
        .or_else(|| fields::name(call))
}

fn call_args(call: &Value) -> Option<String> {
    fields::function(call)
        .and_then(|func| fields::arguments(&func))
        .or_else(|| fields::arguments(call))
}
