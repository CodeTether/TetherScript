//! Exit-status conversion and diagnostics.

use std::collections::HashMap;
use std::io;
use std::process::ExitStatus;

use crate::value::Value;

pub(super) fn value(status: ExitStatus) -> Value {
    let mut fields = HashMap::new();
    fields.insert("success".into(), Value::Bool(status.success()));
    fields.insert(
        "code".into(),
        status
            .code()
            .map_or(Value::Nil, |code| Value::Int(code as i64)),
    );
    Value::Map(std::rc::Rc::new(std::cell::RefCell::new(fields)))
}

pub(super) fn wait_error(error: io::Error) -> String {
    format!("child_process.wait: {error}")
}

pub(super) fn kill_error(error: io::Error) -> String {
    format!("child_process.cancel: {error}")
}
