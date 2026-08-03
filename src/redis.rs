//! Native Redis client speaking RESP over TCP.
//!
//! No driver dependency: this speaks the Redis serialization protocol directly over
//! `std::net::TcpStream`, exactly as [`crate::postgres`] speaks the PostgreSQL v3
//! protocol. The reference application uses Redis for its session store, render
//! cache, and rate-limit counters, none of which the port could reach before.
//!
//! # Layers
//!
//! | Concern | Modules |
//! |---|---|
//! | RESP value model | `value*` |
//! | Encoding a command | `encode*`, `request` |
//! | Decoding a reply | `decode*` |
//! | Connection and handshake | `connection*`, `handshake`, `config*` |
//! | Commands | `commands_*`, `options`, `ttl` |
//! | Bounds and errors | `limits`, `error*` |
//!
//! Every argument leaves as a length-prefixed bulk string, which is what makes
//! command injection structurally impossible rather than a matter of escaping.
//!
//! # Examples
//!
//! ```rust,no_run
//! use tetherscript::redis::{Config, Connection};
//!
//! # fn main() -> Result<(), tetherscript::redis::RedisError> {
//! let mut connection = Connection::connect(&Config::default())?;
//! # Ok(())
//! # }
//! ```

// A second, independently written client: a sibling agent built this as a self-contained
// layer with its own Config, Pool, and codec. Both are kept for now because each has its
// own passing test suite; consolidating them is open work.
#[path = "redis/client.rs"]
pub mod client;
#[path = "redis/commands_get.rs"]
mod commands_get;
#[path = "redis/commands_incr.rs"]
mod commands_incr;
#[path = "redis/commands_key.rs"]
mod commands_key;
#[path = "redis/commands_keyed.rs"]
mod commands_keyed;
#[path = "redis/commands_ping.rs"]
mod commands_ping;
#[path = "redis/commands_set.rs"]
mod commands_set;
#[path = "redis/commands_set_args.rs"]
mod commands_set_args;
#[path = "redis/commands_str.rs"]
mod commands_str;
#[path = "redis/commands_ttl.rs"]
mod commands_ttl;
#[path = "redis/config.rs"]
mod config;
#[path = "redis/config_default.rs"]
mod config_default;
#[path = "redis/connection.rs"]
mod connection;
#[path = "redis/connection_socket.rs"]
mod connection_socket;
#[path = "redis/decode.rs"]
mod decode;
#[path = "redis/decode_array.rs"]
mod decode_array;
#[path = "redis/decode_bulk.rs"]
mod decode_bulk;
#[path = "redis/decode_frame.rs"]
mod decode_frame;
#[path = "redis/decode_int.rs"]
mod decode_int;
#[path = "redis/decode_line.rs"]
mod decode_line;
#[path = "redis/encode.rs"]
mod encode;
#[path = "redis/encode_command.rs"]
mod encode_command;
#[path = "redis/error.rs"]
mod error;
#[path = "redis/error_impl.rs"]
mod error_impl;
#[path = "redis/handler.rs"]
mod handler;
#[path = "redis/handler_command.rs"]
mod handler_command;
#[path = "redis/handler_counter.rs"]
mod handler_counter;
#[path = "redis/handler_exec.rs"]
mod handler_exec;
#[path = "redis/handler_expiry.rs"]
mod handler_expiry;
#[path = "redis/handler_strings.rs"]
mod handler_strings;
#[path = "redis/handler_value.rs"]
mod handler_value;
#[path = "redis/handler_value_resp3.rs"]
mod handler_value_resp3;
#[path = "redis/handshake.rs"]
mod handshake;
#[path = "redis/limits.rs"]
mod limits;
#[path = "redis/options.rs"]
mod options;
#[path = "redis/pool.rs"]
mod pool;
#[path = "redis/pool_lease.rs"]
mod pool_lease;
#[path = "redis/request.rs"]
mod request;
#[path = "redis/round_trip.rs"]
mod round_trip;
#[path = "redis/ttl.rs"]
mod ttl;
#[path = "redis/value.rs"]
mod value;
#[path = "redis/value_bulk.rs"]
mod value_bulk;
#[path = "redis/value_impl.rs"]
mod value_impl;
#[path = "redis/value_int.rs"]
mod value_int;

pub use config::Config;
pub use connection::Connection;
pub use decode::{decode, Decoded};
pub use encode_command::encode_command;
pub use error::RedisError;
pub use handler::RedisHandler;
pub use options::SetOptions;
pub use ttl::Ttl;
pub use value::RespValue;
