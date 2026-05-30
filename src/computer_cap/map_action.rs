//! Standard computer action mapping.

use crate::value::Value;

use super::super::call::ComputerCall;
use super::super::value::{map_value, str_value};

pub(crate) fn prepare(method: &str, args: &[Value]) -> Result<ComputerCall, String> {
    let payload = match args {
        [] => map_value(vec![("action", str_value(method))]),
        [Value::Map(m)] => {
            let mut entries = vec![("action".to_string(), str_value(method))];
            entries.extend(m.borrow().iter().map(|(k, v)| (k.clone(), v.clone())));
            super::super::value::owned_map(entries)
        }
        _ => {
            return Err(format!(
                "computer.{} expects zero args or one params map",
                method
            ))
        }
    };
    super::scoped(method, payload)
}
