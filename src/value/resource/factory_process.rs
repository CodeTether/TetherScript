//! Language factories for supervised child processes.

use crate::value::Value;

use super::{args, factory, OwnedResource};

fn values(values: &[Value], capacity: usize) -> Value {
    let command = args::string(&values[0], "resource.child_process command");
    let arguments = args::strings(&values[1], "resource.child_process arguments");
    factory::resource(command.and_then(|command| {
        arguments.and_then(|arguments| {
            OwnedResource::child_process_bounded(&command, &arguments, capacity)
        })
    }))
}

pub(super) fn child_default(input: &[Value]) -> Result<Value, String> {
    Ok(values(input, 64 * 1024))
}

pub(super) fn child(input: &[Value]) -> Result<Value, String> {
    let capacity = args::usize(&input[2], "resource.child_process_bounded capacity");
    Ok(match capacity {
        Ok(capacity) => values(input, capacity),
        Err(error) => factory::resource(Err(error)),
    })
}
