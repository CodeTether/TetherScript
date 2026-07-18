//! Process discovery and termination built-ins for trusted scripts.
//!
//! `process_list()` returns PID/name maps. `process_kill(pid[, force])`
//! requests graceful or forceful termination through the host operating system.

mod install;
mod kill;
mod parse;
mod platform;

#[cfg(test)]
mod tests;

use std::rc::Rc;

use crate::value::{ResultValue, Value};

pub(crate) use install::install;

fn list() -> Value {
    result(
        platform::list()
            .and_then(parse::processes)
            .map(parse::value),
    )
}

fn kill(args: &[Value]) -> Value {
    result(kill::execute(args))
}

fn result(value: Result<Value, String>) -> Value {
    Value::Result(Rc::new(
        value.map_or_else(ResultValue::Err, ResultValue::Ok),
    ))
}
