//! Minimal blocking HTTP/1.1 server and client.
//!
//! This module re-exports the public API from focused sub-modules:
//!
//! - [`http_client`] — `get`, `head`, `post`, `request` functions
//! - [`http_server`] — `serve` function for the accept loop
//! - [`http_url`] — URL parsing for `http://` scheme
//!
//! Internal sub-modules handle headers, response serialization,
//! response extraction, and status reason phrases.

#[path = "http_client.rs"]
mod http_client;
#[path = "http_headers.rs"]
mod http_headers;
#[path = "http_response.rs"]
mod http_response;
#[path = "http_response_extract.rs"]
mod http_response_extract;
#[path = "http_server.rs"]
mod http_server;
#[path = "http_static/mod.rs"]
mod http_static;
#[path = "http_status.rs"]
mod http_status;
#[path = "http_url.rs"]
mod http_url;

pub use http_client::{get, head, post, request};
pub use http_server::serve;
pub(crate) use http_static::serve as serve_static;

#[cfg(test)]
#[path = "http_tests.rs"]
mod tests;
