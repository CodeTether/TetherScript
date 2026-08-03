//! [`Authority`] dispatch for the `redis` capability.
//!
//! One concern: routing a method name to its body, and nothing else. Every command
//! lives in a sibling `methods_*` module, so this file stays a table that can be
//! read against the documented method list.
//!
//! # Narrowing is refused, not faked
//!
//! `narrow` returns an error. Attenuating Redis authority would mean restricting it
//! to a key prefix or a read-only subset, and the load-bearing invariant in
//! [`crate::capability`] is that a narrowed authority never grants more than its
//! parent. A prefix filter that a script could sidestep — by a binary key, or by any
//! command this capability later gains — would silently violate that, so no
//! narrowing is offered until it can be enforced. `db` refuses for the same reason.
//!
//! # A revoked capability cannot reach the socket
//!
//! [`Capability::invoke`](crate::capability::Capability::invoke) checks the
//! revocation flags before calling into here, so nothing below runs for a revoked
//! grant.

use std::any::Any;
use std::rc::Rc;

use super::RedisAuthority;
use super::{methods_get, methods_incr, methods_key, methods_ping, methods_set, methods_ttl};
use crate::capability::Authority;
use crate::value::{Runtime, Value};

/// Every method this capability exposes, for the unsupported-method message.
const METHODS: &str = "get, set, setex, setnx, del, exists, incr, incrby, expire, ttl, ping";

impl Authority for RedisAuthority {
    /// Refuse attenuation; see the module documentation.
    ///
    /// # Errors
    ///
    /// Always. A narrowing that cannot be enforced is worse than none.
    fn narrow(&self, _params: &Value) -> Result<Rc<dyn Authority>, String> {
        Err("redis: authority does not support narrowing; \
             grant a separate connection with its own database index instead"
            .into())
    }

    /// Dispatch one `redis.*` call.
    ///
    /// # Arguments
    ///
    /// * `method` — Method name without the `redis.` prefix.
    /// * `arguments` — Positional arguments as supplied by the script.
    ///
    /// # Returns
    ///
    /// A [`Value::Result`] in every success case, so a script uses `?` or `match`.
    ///
    /// # Errors
    ///
    /// Returns a usage `Err` for an unknown method, a wrong arity, a badly typed
    /// argument, or a non-positive TTL. Server and transport failures are catchable
    /// `Result::Err` *values* instead; see [`super::outcome`] for the split.
    fn invoke(
        &self,
        _runtime: &mut dyn Runtime,
        method: &str,
        arguments: &[Value],
    ) -> Result<Value, String> {
        let mut connection = self
            .connection
            .try_borrow_mut()
            .map_err(|_| format!("redis.{method}: connection is already in use"))?;
        let connection = &mut connection;
        match method {
            "get" => methods_get::get(connection, arguments),
            "set" => methods_set::set(connection, arguments),
            "setex" => methods_set::setex(connection, arguments),
            "setnx" => methods_set::setnx(connection, arguments),
            "del" => methods_key::del(connection, arguments),
            "exists" => methods_key::exists(connection, arguments),
            "incr" => methods_incr::incr(connection, arguments),
            "incrby" => methods_incr::incrby(connection, arguments),
            "expire" => methods_ttl::expire(connection, arguments),
            "ttl" => methods_ttl::ttl(connection, arguments),
            "ping" => methods_ping::ping(connection, arguments),
            other => Err(format!(
                "redis: unsupported method `{other}` (have: {METHODS})"
            )),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
