//! Language method dispatch for supervised processes.

use std::time::Instant;

use crate::value::Value;

use super::{
    super::{args, result},
    wait, Handle,
};

impl Handle {
    pub(in crate::value::resource) fn call(
        &mut self,
        name: &str,
        values: &[Value],
        deadline: Option<Instant>,
    ) -> Result<Value, String> {
        let value = match (name, values) {
            ("id", []) => Value::Int(self.child.id() as i64),
            ("try_wait", []) => result::value(wait::try_once(&mut self.child)),
            ("wait", []) => result::value(wait::until_exit(&mut self.child, deadline)),
            ("kill", []) => result::nil(self.cancel()),
            ("write_stdin", [value]) => result::value(
                args::bytes(value, "child_process.write_stdin")
                    .and_then(|bytes| self.stdin.try_write(&bytes, "child_process.write_stdin"))
                    .map(|count| Value::Int(count as i64)),
            ),
            ("close_stdin", []) => {
                self.stdin.close();
                result::nil(Ok(()))
            }
            ("read_stdout", [limit]) => self.read_stream(limit, true),
            ("read_stderr", [limit]) => self.read_stream(limit, false),
            ("stdout_eof", []) => Value::Bool(self.stdout.is_eof()),
            ("stderr_eof", []) => Value::Bool(self.stderr.is_eof()),
            ("stream_capacity", []) => Value::Int(self.stdout.capacity() as i64),
            _ => {
                return Err(format!(
                    "child_process: no method `{name}` accepting {} arguments",
                    values.len()
                ))
            }
        };
        Ok(value)
    }
}
