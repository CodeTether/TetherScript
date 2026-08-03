//! Script-facing Redis capability.
//!
//! Grants `redis` to a tetherscript program the same way `db` grants SQL: undefined unless
//! explicitly granted, so a script has no ambient cache or session access. This is the
//! capability shell over [`crate::redis`], which owns the wire protocol.
//!
//! # Layers
//!
//! | Concern | Modules |
//! |---|---|
//! | Capability object | `authority`, `invoke` |
//! | Argument coercion | `args*`, `coerce_*` |
//! | Method dispatch | `methods_*` |
//! | Reply conversion | `reply*`, `outcome` |
//! | Connection URL | `url*` |
//!
//! # Security posture
//!
//! A password in a connection URL must never reach a `Debug` impl, an error message, or a
//! log line — the URL parser strips credentials before anything else can see them. Every
//! command argument goes out as a length-prefixed bulk string, so a key or value containing
//! CRLF cannot forge a second command.

#[path = "redis_cap/args.rs"]
pub mod args;
#[path = "redis_cap/args_error.rs"]
pub mod args_error;
#[path = "redis_cap/args_missing.rs"]
pub mod args_missing;
#[path = "redis_cap/authority.rs"]
pub mod authority;
#[path = "redis_cap/coerce_bytes.rs"]
pub mod coerce_bytes;
#[path = "redis_cap/coerce_int.rs"]
pub mod coerce_int;
#[path = "redis_cap/coerce_seconds.rs"]
pub mod coerce_seconds;
#[path = "redis_cap/invoke.rs"]
mod invoke;
#[path = "redis_cap/methods_get.rs"]
mod methods_get;
#[path = "redis_cap/methods_incr.rs"]
mod methods_incr;
#[path = "redis_cap/methods_key.rs"]
mod methods_key;
#[path = "redis_cap/methods_key_args.rs"]
pub mod methods_key_args;
#[path = "redis_cap/methods_ping.rs"]
mod methods_ping;
#[path = "redis_cap/methods_set.rs"]
mod methods_set;
#[path = "redis_cap/methods_ttl.rs"]
mod methods_ttl;
#[path = "redis_cap/outcome.rs"]
pub mod outcome;
#[path = "redis_cap/reply.rs"]
pub mod reply;
#[path = "redis_cap/reply_ttl.rs"]
pub mod reply_ttl;
#[path = "redis_cap/url.rs"]
pub mod url;
#[path = "redis_cap/url_credentials.rs"]
pub mod url_credentials;
#[path = "redis_cap/url_db.rs"]
pub mod url_db;
#[path = "redis_cap/url_port.rs"]
pub mod url_port;

// The coercion helpers are reached as modules (`redis_cap::args::…`) rather than
// re-exported flat, matching how the tests address them.
pub use authority::RedisAuthority;
