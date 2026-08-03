//! `redis.ping` — liveness.
//!
//! One concern: confirming the connection is usable. Separate from the data commands
//! because it takes no key and is the one method a health check calls.

use std::rc::Rc;

use crate::redis::Connection;
use crate::redis_cap::{args, outcome};
use crate::value::Value;

/// `redis.ping()` — check that the server answers.
///
/// # Arguments
///
/// * `arguments` — Must be empty; an extra argument is refused rather than ignored.
///
/// # Returns
///
/// `Ok(Result::Ok("PONG"))`. The status line is returned rather than a bool so a failing
/// health check can log what actually came back.
///
/// # Errors
///
/// A usage `Err` when any argument is supplied. A dead socket is a catchable
/// `Result::Err`, which is the point: a health check must be able to observe failure
/// without being aborted by it.
pub(super) fn ping(connection: &mut Connection, arguments: &[Value]) -> Result<Value, String> {
    args::exactly("redis.ping", arguments, 0)?;
    match connection.ping() {
        Ok(status) => Ok(outcome::ok(Value::Str(Rc::new(status)))),
        Err(error) => Ok(outcome::failed("redis.ping", error)),
    }
}
