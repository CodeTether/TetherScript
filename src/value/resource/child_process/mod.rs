//! Owned child-process handles.

mod spawn;
mod status;
mod wait;

use std::process::Child;
use std::time::Instant;

use crate::value::Value;

use super::result;

pub(super) struct Handle {
    child: Child,
}

impl Handle {
    pub(super) fn spawn(command: &str, args: &[String]) -> Result<Self, String> {
        spawn::child(command, args).map(|child| Self { child })
    }

    pub(super) fn call(
        &mut self,
        name: &str,
        args: &[Value],
        deadline: Option<Instant>,
    ) -> Result<Value, String> {
        match (name, args) {
            ("id", []) => Ok(Value::Int(self.child.id() as i64)),
            ("try_wait", []) => Ok(result::value(wait::try_once(&mut self.child))),
            ("wait", []) => Ok(result::value(wait::until_exit(&mut self.child, deadline))),
            ("kill", []) => Ok(result::nil(self.cancel())),
            _ => Err(format!(
                "child_process: no method `{name}` accepting {} arguments",
                args.len()
            )),
        }
    }

    pub(super) fn cancel(&mut self) -> Result<(), String> {
        if self.child.try_wait().map_err(status::wait_error)?.is_none() {
            self.child.kill().map_err(status::kill_error)?;
            self.child.wait().map_err(status::wait_error)?;
        }
        Ok(())
    }
}
