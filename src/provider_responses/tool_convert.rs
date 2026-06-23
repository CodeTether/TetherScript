//! Responses function tool shape conversion.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

pub(super) fn convert(value: &Value) -> Result<Value, String> {
    let Value::Map(tool) = value else {
        return Err("provider.responses: tool definition must be a map".into());
    };
    let tool = tool.borrow();
    match (function_type(&tool), tool.get("function")) {
        (true, Some(Value::Map(function))) => Ok(flat_function(function)),
        _ => Ok(value.clone()),
    }
}

fn function_type(map: &HashMap<String, Value>) -> bool {
    matches!(map.get("type"), Some(Value::Str(text)) if text.as_str() == "function")
}

fn flat_function(function: &Rc<RefCell<HashMap<String, Value>>>) -> Value {
    let mut out = function.borrow().clone();
    out.insert("type".into(), Value::Str(Rc::new("function".into())));
    Value::Map(Rc::new(RefCell::new(out)))
}
