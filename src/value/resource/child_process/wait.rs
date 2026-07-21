//! Deadline-aware child waiting.

use std::process::Child;
use std::thread;
use std::time::{Duration, Instant};

use crate::value::Value;

use super::status;

pub(super) fn try_once(child: &mut Child) -> Result<Value, String> {
    child
        .try_wait()
        .map(|status| status.map_or(Value::Nil, status::value))
        .map_err(status::wait_error)
}

pub(super) fn until_exit(child: &mut Child, deadline: Option<Instant>) -> Result<Value, String> {
    if deadline.is_none() {
        return child.wait().map(status::value).map_err(status::wait_error);
    }
    loop {
        if let Some(status) = child.try_wait().map_err(status::wait_error)? {
            return Ok(status::value(status));
        }
        if deadline.is_some_and(|instant| Instant::now() >= instant) {
            return Err("child_process.wait: deadline exceeded".into());
        }
        thread::sleep(Duration::from_millis(1));
    }
}
