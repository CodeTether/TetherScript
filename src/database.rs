//! Host-neutral database capabilities for embedded tetherscript programs.
//!
//! [`DatabaseAuthority`] exposes `db.query(sql, parameters)` while a host-owned
//! [`QueryHandler`] performs the actual database work. HTTP frameworks and
//! database drivers therefore depend on this contract, never the reverse.
//!
//! # Usage
//!
//! ```
//! use tetherscript::database::{DatabaseAuthority, QueryHandler};
//! use tetherscript::value::Value;
//!
//! struct MemoryDatabase;
//! impl QueryHandler for MemoryDatabase {
//!     fn query(&self, _sql: &str, _parameters: &[Value]) -> Result<Value, String> {
//!         Ok(Value::Nil)
//!     }
//! }
//! let _authority = DatabaseAuthority::new(MemoryDatabase);
//! ```

mod authority;
mod invoke;
mod query;
mod query_handler;

pub use authority::DatabaseAuthority;
pub use query_handler::QueryHandler;
