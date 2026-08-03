//! # Wiring this module into the crate
//!
//! This file contains no code. It exists because the client is delivered as
//! `src/redis/*.rs` without a `src/redis.rs` or `src/redis/mod.rs`: the integrator
//! owns that file, so the exact declarations it needs are recorded here rather than
//! left to be rediscovered.
//!
//! ## What `src/redis.rs` must contain
//!
//! ```rust,ignore
//! mod commands_get;
//! mod commands_incr;
//! mod commands_key;
//! mod commands_keyed;
//! mod commands_ping;
//! mod commands_set;
//! mod commands_set_args;
//! mod commands_str;
//! mod commands_ttl;
//! mod config;
//! mod config_default;
//! mod connection;
//! mod connection_socket;
//! mod decode;
//! mod decode_array;
//! mod decode_bulk;
//! mod decode_frame;
//! mod decode_int;
//! mod decode_line;
//! mod encode;
//! mod encode_command;
//! mod error;
//! mod error_impl;
//! mod handshake;
//! mod limits;
//! mod options;
//! mod request;
//! mod round_trip;
//! mod ttl;
//! mod value;
//! mod value_bulk;
//! mod value_impl;
//! mod value_int;
//! mod wiring;
//!
//! pub use config::Config;
//! pub use connection::Connection;
//! pub use decode::{Decoded, decode};
//! pub use encode_command::encode_command;
//! pub use error::RedisError;
//! pub use options::SetOptions;
//! pub use ttl::Ttl;
//! pub use value::RespValue;
//! ```
//!
//! Add `pub mod redis;` to `src/lib.rs`. Nothing else changes: the client has no
//! dependencies, no feature flag, and no `Cargo.toml` entry, since it speaks RESP
//! over `std::net::TcpStream` exactly as `src/postgres/` speaks the v3 protocol.
//!
//! ## Public surface
//!
//! | Item | Purpose |
//! |---|---|
//! | `Config`, `Config::from_address` | Address, credentials, database, timeouts |
//! | `Connection` | Connect, handshake, and command methods |
//! | `encode_command` | The only supported way to build a request |
//! | `decode`, `Decoded` | Incremental-safe reply decoding |
//! | `RespValue` | The RESP value model, nulls kept distinct |
//! | `RedisError` | Transport / protocol / server / unexpected-type |
//! | `SetOptions`, `Ttl` | `SET` modifiers and the three `TTL` states |
//!
//! ## Not included, deliberately
//!
//! - **No capability object.** Granting `redis` to scripts is the host's decision,
//!   and the `fs`/`db` precedent is that the capability wrapper lives beside the
//!   authority model, not inside the wire client.
//! - **No connection pool.** `src/postgres/pool.rs` is the pattern to copy when one
//!   is wanted; a Redis round trip is short enough that a single connection is the
//!   honest default until a pool is measured to be needed.
//! - **No TLS.** As with the PostgreSQL client, connections are cleartext. Use a
//!   trusted network or a tunnel.
//! - **No pipelining or pub/sub.** The decoder already returns a `consumed` count
//!   so pipelining can be added without changing the codec.
