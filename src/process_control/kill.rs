use crate::value::Value;

use super::platform;

pub(super) fn execute(args: &[Value]) -> Result<Value, String> {
    if !(1..=2).contains(&args.len()) {
        return Err("process_kill expects pid[, force]".into());
    }
    let Value::Int(pid) = args[0] else {
        return Err(format!(
            "process_kill: pid must be int, got {}",
            args[0].type_name()
        ));
    };
    if !(1..=i32::MAX as i64).contains(&pid) {
        return Err("process_kill: pid must be between 1 and 2147483647".into());
    }
    let force = force(args.get(1))?;
    platform::kill(pid, force).map(|()| Value::Nil)
}

fn force(value: Option<&Value>) -> Result<bool, String> {
    match value {
        None => Ok(false),
        Some(Value::Bool(force)) => Ok(*force),
        Some(value) => Err(format!(
            "process_kill: force must be bool, got {}",
            value.type_name()
        )),
    }
}
