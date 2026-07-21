//! Process stream value conversion.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Value;

use super::{
    super::{args, result},
    Handle,
};

impl Handle {
    pub(super) fn read_stream(&self, limit: &Value, stdout: bool) -> Value {
        let label = if stdout {
            "child_process.read_stdout"
        } else {
            "child_process.read_stderr"
        };
        let buffer = if stdout { &self.stdout } else { &self.stderr };
        result::value(
            args::usize(limit, label)
                .and_then(|limit| buffer.read(limit, label))
                .map(|bytes| Value::Bytes(Rc::new(RefCell::new(bytes)))),
        )
    }
}
