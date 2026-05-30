//! Raw computer action mapping.

use crate::value::Value;

use super::super::call::ComputerCall;
use super::super::value::with_action;

pub(crate) fn prepare(args: &[Value]) -> Result<ComputerCall, String> {
    let action = match args.first() {
        Some(Value::Str(s)) => s.as_str(),
        Some(other) => {
            return Err(format!(
                "computer.raw action must be str, got {}",
                other.type_name()
            ))
        }
        None => return Err("computer.raw: missing action".into()),
    };
    let params = args.get(1).unwrap_or(&Value::Nil);
    super::scoped(action, with_action(action, params)?)
}
