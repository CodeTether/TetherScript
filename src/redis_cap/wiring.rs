//! # Wiring this capability into the crate
//!
//! This file contains no code. The capability ships as `src/redis_cap/*.rs` with no
//! `src/redis_cap.rs` and no `mod.rs`, because the integrator owns that file. The exact
//! declarations it needs are recorded here rather than left to be rediscovered, matching
//! the convention in [`crate::redis`]'s own `wiring` module.
//!
//! ## What `src/redis_cap.rs` must contain
//!
//! ```rust,ignore
//! //! The `redis` capability: `RedisAuthority` and the `--grant-redis` URL parser.
//!
//! mod args;
//! mod args_error;
//! mod args_missing;
//! mod authority;
//! mod coerce_bytes;
//! mod coerce_int;
//! mod coerce_seconds;
//! mod invoke;
//! mod methods_get;
//! mod methods_incr;
//! mod methods_key;
//! mod methods_key_args;
//! mod methods_ping;
//! mod methods_set;
//! mod methods_ttl;
//! mod outcome;
//! mod reply;
//! mod reply_ttl;
//! mod url;
//! mod url_credentials;
//! mod url_db;
//! mod url_port;
//! mod wiring;
//!
//! pub use authority::RedisAuthority;
//! ```
//!
//! The `args*`, `coerce_*`, `outcome`, `reply*`, and `url*` modules are `pub` inside
//! their files because `tests/redis_cap.rs`, `tests/grant_redis_url.rs`, and their
//! siblings exercise them directly. Declare them `pub mod` rather than `mod` to keep
//! those integration tests compiling:
//!
//! ```rust,ignore
//! pub mod args;
//! pub mod args_error;
//! pub mod args_missing;
//! pub mod coerce_bytes;
//! pub mod coerce_int;
//! pub mod coerce_seconds;
//! pub mod outcome;
//! pub mod reply;
//! pub mod reply_ttl;
//! pub mod url;
//! pub mod url_credentials;
//! pub mod url_db;
//! pub mod url_port;
//!
//! mod authority;
//! mod invoke;
//! mod methods_get;
//! mod methods_incr;
//! mod methods_key;
//! pub mod methods_key_args;
//! mod methods_ping;
//! mod methods_set;
//! mod methods_ttl;
//! mod wiring;
//!
//! pub use authority::RedisAuthority;
//! ```
//!
//! Add `pub mod redis_cap;` to `src/lib.rs` and `mod redis_cap;` to `src/main.rs`,
//! alongside the existing `fs_cap` and `database` entries.
//!
//! ## CLI wiring
//!
//! `src/main_caps/mod.rs` gains:
//!
//! ```rust,ignore
//! mod redis;
//! mod redis_url;
//! ```
//!
//! and `RunCaps` gains `pub redis_grant: &'a Option<String>`. Both
//! `src/main_caps/vm.rs` and `src/main_caps/interp.rs` then grant it exactly as they
//! grant `db`:
//!
//! ```rust,ignore
//! if let Some(auth) = redis::authority(caps.redis_grant)? {
//!     vm.grant("redis", std::rc::Rc::new(auth));
//! }
//! ```
//!
//! `src/main.rs` gains a `--grant-redis <url>` arm mirroring `--grant-db`, requiring a
//! value and erroring with `--grant-redis requires a redis:// URL` when absent.
//!
//! ## Not included, deliberately
//!
//! - **No `--access-mode full` implication.** A Redis URL carries credentials and a
//!   database index that cannot be guessed, so the grant is always explicit. Same
//!   judgement as `--grant-db`.
//! - **No ambient `redis_*` builtin.** AGENTS.md records `fs_*` and `process_*` as
//!   bypassing capability grants; there is deliberately no equivalent hole here, so the
//!   capability is the only path and nothing has to be closed later.
//! - **No narrowing.** See [`super::invoke`]: a key-prefix restriction a script could
//!   sidestep would violate the "narrowed ⊆ parent" invariant in
//!   [`crate::capability`], so none is offered until it can be enforced.
//! - **No connection pool.** [`RedisAuthority`](super::RedisAuthority) holds one
//!   [`Connection`](crate::redis::Connection) in a `RefCell`. The runtime is
//!   single-threaded and a Redis round trip is short; `src/postgres/pool.rs` is the
//!   pattern to copy if one is measured to be needed.
//! - **No TLS.** [`super::url`] refuses `rediss://` rather than downgrading.
//!
//! ## Blocker for the integrator
//!
//! `src/redis/` as delivered contains two incompatible drafts. The `Connection`-based
//! set listed in `src/redis/wiring.rs` is authoritative and is what this capability is
//! built on. The sibling `handler*.rs`, `pool*.rs`, `handler_value*.rs`, and
//! `probe_write.rs` files are **not** in that list and will not compile: they reference
//! `super::resp::Resp` (no `src/redis/resp.rs` exists) and construct `Config { tls: … }`
//! (no such field). Omit those `mod` declarations from `src/redis.rs`.
