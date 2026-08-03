//! # Redis client and connection pool
//!
//! A blocking Redis client: TCP connect, optional `AUTH`, optional `SELECT`, a
//! set of typed commands, a generic escape hatch, and a lazily grown connection
//! pool. It is the sibling of [`crate::postgres`]: same synchronous shape, same
//! lease/return/discard pool, no dependencies.
//!
//! ## Why this module contains no RESP codec
//!
//! Framing RESP is a separate concern owned by a separate module. This client
//! reaches a codec only through [`codec::RespCodec`], a two-method boundary, and
//! the network only through [`transport::Transport`]. Both are injected, so every
//! behaviour below is exercised with no server and no real codec; see
//! `tests/redis_client.rs`.
//!
//! ## Wiring
//!
//! The integrator owns `src/redis.rs`. To expose this tree, add:
//!
//! ```rust,ignore
//! #[path = "redis/client.rs"]
//! pub mod client;
//! ```
//!
//! plus one adapter implementing [`codec::RespCodec`] over the real RESP codec.
//! [`codec::RespCodec`] documents that adapter in full.
//!
//! ## Quick start
//!
//! ```rust,ignore
//! use tetherscript::redis::client::{Config, Connection, Pool};
//!
//! let config = Config::new("127.0.0.1", 6379);
//! let open = |c: &Config| Connection::connect(c, Box::new(RespAdapter));
//! let pool = Pool::new(config, 8, Box::new(open));
//! let cached: Option<Vec<u8>> = pool.with_connection(|c| c.get(b"render:home"))?;
//! ```
//!
//! ## Architecture
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`config`] | Address, credentials, database index, timeouts |
//! | [`error`] | Error taxonomy and the discard-versus-reuse verdict |
//! | [`reply`] | The reply value model and its typed accessors |
//! | [`codec`] | The encode-command / decode-reply boundary |
//! | [`transport`] | The byte-stream boundary and its TCP implementation |
//! | [`connection`] | Connect, handshake, exchange, typed commands |
//! | [`ttl`] | The three states of a key's time-to-live |
//! | [`pool`] | Lazily grown pool: acquire, release, discard |

#[path = "client_codec.rs"]
pub mod codec;
#[path = "client_config.rs"]
pub mod config;
#[path = "client_connection.rs"]
pub mod connection;
#[path = "client_error.rs"]
pub mod error;
#[path = "client_pool.rs"]
pub mod pool;
#[path = "client_reply.rs"]
pub mod reply;
#[path = "client_transport.rs"]
pub mod transport;
#[path = "client_ttl.rs"]
pub mod ttl;

#[path = "client_cmd_counter.rs"]
mod cmd_counter;
#[path = "client_cmd_expire.rs"]
mod cmd_expire;
#[path = "client_cmd_key.rs"]
mod cmd_key;
#[path = "client_cmd_string.rs"]
mod cmd_string;
#[path = "client_config_debug.rs"]
mod config_debug;
#[path = "client_connect.rs"]
mod connect;
#[path = "client_exchange.rs"]
mod exchange;
#[path = "client_handshake.rs"]
mod handshake;
#[path = "client_pool_lease.rs"]
mod pool_lease;
#[path = "client_pool_with.rs"]
mod pool_with;
#[path = "client_reply_access.rs"]
mod reply_access;

pub use codec::RespCodec;
pub use config::Config;
pub use connection::Connection;
pub use error::ClientError;
pub use pool::{Connector, Pool};
pub use reply::Reply;
pub use transport::Transport;
pub use ttl::Ttl;
